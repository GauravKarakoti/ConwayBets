import { lineraAdapter } from './linera-adapter';
import { EventEmitter } from 'events';

// Define interfaces locally as they are not exported by @linera/client
interface LineraQuery {
  query: string;
  variables?: Record<string, any>;
}

interface LineraMutation {
  mutation: string;
  variables?: Record<string, any>;
}

export interface Market {
  id: string;
  title: string;
  description: string;
  creator: string;
  endTime: number;
  outcomes: string[];
  totalLiquidity: string;
  isResolved: boolean;
  winningOutcome?: number;
  stateHash: string;
  createdAt: number;
}

export interface Bet {
  id: string;
  marketId: string;
  user: string;
  outcomeIndex: number;
  amount: string;
  odds: string;
  placedAt: number;
  status: 'pending' | 'confirmed' | 'resolved';
}

export interface UserPortfolio {
  totalValue: string;
  activeBets: number;
  resolvedBets: number;
  totalProfit: string;
  positions: PortfolioPosition[];
}

export interface PortfolioPosition {
  marketId: string;
  marketTitle: string;
  outcomeIndex: number;
  outcomeLabel: string;
  amount: string;
  currentValue: string;
  potentialProfit: string;
}

export class ConwayBetsClient {
  private eventEmitter: EventEmitter;
  private pollingInterval: NodeJS.Timeout | null = null;

  constructor(private endpoint: string, private applicationId: string) {
    this.eventEmitter = new EventEmitter();
  }

  async connect(): Promise<boolean> {
    try {
      // Connection is handled by LineraAdapter/WalletConnector.
      // We just ensure the application is set if connected.
      if (lineraAdapter.isChainConnected()) {
        if (!lineraAdapter.isApplicationSet()) {
          await lineraAdapter.setApplication(this.applicationId);
        }
        console.log('ConwayBetsClient connected via Adapter');
        return true;
      }
      return false;
    } catch (error) {
      console.error('Failed to connect to Linera application:', error);
      return false;
    }
  }

  async getAllMarkets(limit: number = 50, offset: number = 0): Promise<Market[]> {
    const query: LineraQuery = {
      query: `
        query GetAllMarkets($limit: Int!, $offset: Int!) {
          markets(limit: $limit, offset: $offset) {
            id
            title
            description
            creator
            endTime
            outcomes
            totalLiquidity
            isResolved
            winningOutcome
            stateHash
            createdAt
          }
        }
      `,
      variables: { limit, offset },
    };

    const result = await lineraAdapter.queryApplication<{ data: { markets: Market[] } }>(query);
    return result.data.markets;
  }

  async getMarket(marketId: string): Promise<Market> {
    const query: LineraQuery = {
      query: `
        query GetMarket($marketId: ID!) {
          market(id: $marketId) {
            id
            title
            description
            creator
            endTime
            outcomes
            totalLiquidity
            isResolved
            winningOutcome
            stateHash
            createdAt
          }
        }
      `,
      variables: { marketId },
    };

    const result = await lineraAdapter.queryApplication<{ data: { market: Market } }>(query);
    return result.data.market;
  }

  async createMarket(
    creator: string,
    title: string,
    description: string,
    endTime: number,
    outcomes: string[]
  ): Promise<string> {
    const mutation: LineraMutation = {
      mutation: `
        mutation CreateMarket(
          $creator: String!,
          $title: String!,
          $description: String!,
          $endTime: Int!,
          $outcomes: [String!]!
        ) {
          createMarket(
            creator: $creator,
            title: $title,
            description: $description,
            endTime: $endTime,
            outcomes: $outcomes
          ) {
            marketId
            receipt {
              id
              status
              finalizedAt
            }
          }
        }
      `,
      variables: { creator, title, description, endTime, outcomes },
    };

    // Note: If the backend expects actual operations instead of GraphQL mutations, 
    // this might need adjustment, but we pass the mutation structure to the adapter.
    const result = await lineraAdapter.queryApplication<{ data: { createMarket: { marketId: string } } }>(mutation);
    return result.data.createMarket.marketId;
  }

  async placeBet(
    marketId: string,
    user: string,
    outcomeIndex: number,
    amount: string
  ): Promise<{ receiptId: string; status: string }> {
    const mutation: LineraMutation = {
      mutation: `
        mutation PlaceBet(
          $marketId: ID!,
          $user: String!,
          $outcomeIndex: Int!,
          $amount: String!
        ) {
          placeBet(
            marketId: $marketId,
            user: $user,
            outcomeIndex: $outcomeIndex,
            amount: $amount
          ) {
            receipt {
              id
              status
              finalizedAt
            }
            betId
          }
        }
      `,
      variables: { marketId, user, outcomeIndex, amount },
    };

    const result = await lineraAdapter.queryApplication<{ data: { placeBet: { receipt: { id: string, status: string } } } }>(mutation);
    return {
      receiptId: result.data.placeBet.receipt.id,
      status: result.data.placeBet.receipt.status,
    };
  }

  async getUserPortfolio(userAddress: string): Promise<UserPortfolio> {
    const query: LineraQuery = {
      query: `
        query GetUserPortfolio($userAddress: String!) {
          userPortfolio(address: $userAddress) {
            totalValue
            activeBets
            resolvedBets
            totalProfit
            positions {
              marketId
              marketTitle
              outcomeIndex
              outcomeLabel
              amount
              currentValue
              potentialProfit
            }
          }
        }
      `,
      variables: { userAddress },
    };

    const result = await lineraAdapter.queryApplication<{ data: { userPortfolio: UserPortfolio } }>(query);
    return result.data.userPortfolio;
  }

  // Polling mechanism as fallback/replacement for subscriptions
  startPolling(marketId: string, interval: number = 5000): NodeJS.Timeout {
    if (this.pollingInterval) clearInterval(this.pollingInterval);
    
    this.pollingInterval = setInterval(async () => {
      try {
        const market = await this.getMarket(marketId);
        this.eventEmitter.emit('marketUpdate', market);
      } catch (error) {
        console.error('Polling failed:', error);
      }
    }, interval);
    
    return this.pollingInterval;
  }

  stopPolling() {
    if (this.pollingInterval) {
      clearInterval(this.pollingInterval);
      this.pollingInterval = null;
    }
  }

  on(event: string, listener: (...args: any[]) => void): void {
    this.eventEmitter.on(event, listener);
  }

  off(event: string, listener: (...args: any[]) => void): void {
    this.eventEmitter.off(event, listener);
  }

  disconnect(): void {
    this.stopPolling();
    // Adapter handles actual disconnect
  }
}

// Singleton instance
let instance: ConwayBetsClient | null = null;

export function getConwayBetsClient(): ConwayBetsClient {
  if (!instance) {
    const endpoint = import.meta.env.VITE_LINERA_ENDPOINT || 'https://conway.linera.io';
    const applicationId = import.meta.env.VITE_APPLICATION_ID;
    
    if (!applicationId) {
      throw new Error('VITE_APPLICATION_ID environment variable is required');
    }
    
    instance = new ConwayBetsClient(endpoint, applicationId);
  }
  return instance;
}
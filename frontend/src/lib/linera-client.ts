import { LineraClient, LineraQuery, LineraMutation, Subscription } from '@linera/client';
import { EventEmitter } from 'events';

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
  private client: LineraClient;
  private eventEmitter: EventEmitter;
  private reconnectAttempts: number = 0;
  private maxReconnectAttempts: number = 5;
  private subscription?: Subscription;

  constructor(endpoint: string, private applicationId: string) {
    this.client = new LineraClient({
      endpoint,
      applicationId,
      reconnect: true,
      reconnectInterval: 3000,
    });
    this.eventEmitter = new EventEmitter();
  }

  async connect(): Promise<boolean> {
    try {
      await this.client.connect();
      console.log('Connected to Linera Conway testnet');
      return true;
    } catch (error) {
      console.error('Failed to connect to Linera:', error);
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

    const result = await this.client.query(query);
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

    const result = await this.client.query(query);
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

    const result = await this.client.mutate(mutation);
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

    const result = await this.client.mutate(mutation);
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

    const result = await this.client.query(query);
    return result.data.userPortfolio;
  }

  subscribeToMarketUpdates(marketId: string): Subscription {
    if (this.subscription) {
      this.subscription.unsubscribe();
    }

    this.subscription = this.client.subscribe({
      query: `
        subscription MarketUpdates($marketId: ID!) {
          marketUpdated(id: $marketId) {
            id
            totalLiquidity
            isResolved
            winningOutcome
            stateHash
            updatedAt
          }
        }
      `,
      variables: { marketId },
    });

    return this.subscription;
  }

  subscribeToUserUpdates(userAddress: string): Subscription {
    return this.client.subscribe({
      query: `
        subscription UserUpdates($userAddress: String!) {
          userUpdated(address: $userAddress) {
            totalValue
            activeBets
            positions {
              marketId
              currentValue
            }
          }
        }
      `,
      variables: { userAddress },
    });
  }

  // Fallback polling mechanism for when subscriptions fail
  startPolling(marketId: string, interval: number = 5000): NodeJS.Timeout {
    return setInterval(async () => {
      try {
        const market = await this.getMarket(marketId);
        this.eventEmitter.emit('marketUpdate', market);
      } catch (error) {
        console.error('Polling failed:', error);
      }
    }, interval);
  }

  on(event: string, listener: (...args: any[]) => void): void {
    this.eventEmitter.on(event, listener);
  }

  off(event: string, listener: (...args: any[]) => void): void {
    this.eventEmitter.off(event, listener);
  }

  disconnect(): void {
    if (this.subscription) {
      this.subscription.unsubscribe();
    }
    this.client.disconnect();
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
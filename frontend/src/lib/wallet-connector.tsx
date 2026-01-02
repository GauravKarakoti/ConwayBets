import { useState, useEffect, createContext, useContext } from 'react';

export interface WalletState {
  isConnected: boolean;
  address: string | null;
  chainId: number | null;
  balance: string | null;
  walletName: string | null;
}

export interface WalletError {
  code: string;
  message: string;
  details?: any;
}

const WalletContext = createContext<{
  wallet: WalletState;
  connect: () => Promise<void>;
  disconnect: () => Promise<void>;
  switchNetwork: (chainId: number) => Promise<void>;
  signMessage: (message: string) => Promise<string>;
} | null>(null);

export const useWallet = () => {
  const context = useContext(WalletContext);
  if (!context) {
    throw new Error('useWallet must be used within WalletProvider');
  }
  return context;
};

export class WalletService {
  private dynamicProvider: any = null;
  private walletState: WalletState = {
    isConnected: false,
    address: null,
    chainId: null,
    balance: null,
    walletName: null,
  };
  private eventHandlers: Map<string, Function[]> = new Map();

  constructor() {
    this.initializeDynamic();
  }

  private initializeDynamic() {
    const environmentId = import.meta.env.VITE_DYNAMIC_ENVIRONMENT_ID;
    
    if (!environmentId) {
      console.warn('Dynamic environment ID not found, wallet features disabled');
      return;
    }

    this.dynamicProvider = {
      environmentId,
      theme: 'dark',
      walletConnectProjectId: import.meta.env.VITE_WALLET_CONNECT_PROJECT_ID,
      appName: 'ConwayBets',
      appLogoUrl: 'https://conwaybets.com/logo.png',
      evmNetworks: [
        {
          blockExplorerUrls: ['https://conway-explorer.linera.io'],
          chainId: 1234, // Conway testnet chain ID
          chainName: 'Linera Conway Testnet',
          iconUrls: ['https://linera.io/icon.png'],
          name: 'Conway',
          nativeCurrency: {
            decimals: 18,
            name: 'Test Token',
            symbol: 'TEST',
          },
          networkId: 1234,
          rpcUrls: ['https://faucet.testnet-conway.linera.net'],
          vanityName: 'Conway',
        },
      ],
      walletsFilter: (wallets: any[]) => {
        // Filter to show only supported wallets
        return wallets.filter(wallet =>
          wallet.installed || 
          wallet.downloadUrl || 
          ['metamask', 'coinbase', 'walletconnect'].includes(wallet.key)
        );
      },
      settings: {
        privacyPolicyUrl: 'https://conwaybets.com/privacy',
        termsOfServiceUrl: 'https://conwaybets.com/terms',
      },
    };
  }

  async connect(): Promise<WalletState> {
    try {
      if (!this.dynamicProvider) {
        throw new Error('Wallet provider not initialized');
      }

      // In a real implementation, this would trigger the Dynamic widget
      console.log('Connecting wallet...');
      
      // Simulate connection for demo
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      this.walletState = {
        isConnected: true,
        address: '0x' + Array.from({ length: 40 }, () => 
          Math.floor(Math.random() * 16).toString(16)
        ).join(''),
        chainId: 1234,
        balance: '1000.0',
        walletName: 'MetaMask',
      };

      this.emit('connected', this.walletState);
      return this.walletState;
    } catch (error) {
      console.error('Failed to connect wallet:', error);
      throw error;
    }
  }

  async disconnect(): Promise<void> {
    try {
      this.walletState = {
        isConnected: false,
        address: null,
        chainId: null,
        balance: null,
        walletName: null,
      };
      
      this.emit('disconnected', null);
    } catch (error) {
      console.error('Failed to disconnect wallet:', error);
      throw error;
    }
  }

  async signMessage(message: string): Promise<string> {
    if (!this.walletState.isConnected || !this.walletState.address) {
      throw new Error('Wallet not connected');
    }

    try {
      // In a real implementation, this would use the wallet's signMessage method
      console.log('Signing message:', message);
      
      // Simulate signing for demo
      await new Promise(resolve => setTimeout(resolve, 500));
      
      return '0x' + Array.from({ length: 130 }, () => 
        Math.floor(Math.random() * 16).toString(16)
      ).join('');
    } catch (error) {
      console.error('Failed to sign message:', error);
      throw error;
    }
  }

  async switchNetwork(chainId: number): Promise<void> {
    try {
      console.log('Switching to network:', chainId);
      
      // Update wallet state
      this.walletState.chainId = chainId;
      this.emit('networkChanged', chainId);
    } catch (error) {
      console.error('Failed to switch network:', error);
      throw error;
    }
  }

  async getBalance(): Promise<string> {
    if (!this.walletState.isConnected) {
      throw new Error('Wallet not connected');
    }

    // Simulate balance fetch
    return this.walletState.balance || '0';
  }

  getState(): WalletState {
    return { ...this.walletState };
  }

  on(event: string, handler: Function): void {
    if (!this.eventHandlers.has(event)) {
      this.eventHandlers.set(event, []);
    }
    this.eventHandlers.get(event)!.push(handler);
  }

  off(event: string, handler: Function): void {
    const handlers = this.eventHandlers.get(event);
    if (handlers) {
      const index = handlers.indexOf(handler);
      if (index > -1) {
        handlers.splice(index, 1);
      }
    }
  }

  private emit(event: string, data: any): void {
    const handlers = this.eventHandlers.get(event);
    if (handlers) {
      handlers.forEach(handler => handler(data));
    }
  }

  isAvailable(): boolean {
    return !!this.dynamicProvider;
  }
}

// React component for wallet connection
export function WalletProvider({ children }: { children: React.ReactNode }) {
  const [walletState, setWalletState] = useState<WalletState>({
    isConnected: false,
    address: null,
    chainId: null,
    balance: null,
    walletName: null,
  });

  const [walletService] = useState(() => new WalletService());

  useEffect(() => {
    // Set up event listeners
    walletService.on('connected', (state: WalletState) => {
      setWalletState(state);
    });

    walletService.on('disconnected', () => {
      setWalletState({
        isConnected: false,
        address: null,
        chainId: null,
        balance: null,
        walletName: null,
      });
    });

    walletService.on('networkChanged', (chainId: number) => {
      setWalletState(prev => ({ ...prev, chainId }));
    });

    // Check initial connection
    const checkConnection = async () => {
      const state = walletService.getState();
      if (state.isConnected) {
        setWalletState(state);
      }
    };

    checkConnection();

    return () => {
      // Cleanup
      walletService.off('connected', () => {});
      walletService.off('disconnected', () => {});
      walletService.off('networkChanged', () => {});
    };
  }, [walletService]);

  const connect = async () => {
    try {
      const state = await walletService.connect();
      setWalletState(state);
    } catch (error) {
      console.error('Failed to connect wallet:', error);
      throw error;
    }
  };

  const disconnect = async () => {
    try {
      await walletService.disconnect();
    } catch (error) {
      console.error('Failed to disconnect wallet:', error);
      throw error;
    }
  };

  const switchNetwork = async (chainId: number) => {
    try {
      await walletService.switchNetwork(chainId);
    } catch (error) {
      console.error('Failed to switch network:', error);
      throw error;
    }
  };

  const signMessage = async (message: string) => {
    try {
      return await walletService.signMessage(message);
    } catch (error) {
      console.error('Failed to sign message:', error);
      throw error;
    }
  };

  return (
    <WalletContext.Provider
      value={{
        wallet: walletState,
        connect,
        disconnect,
        switchNetwork,
        signMessage,
      }}
    >
      {children}
    </WalletContext.Provider>
  );
}
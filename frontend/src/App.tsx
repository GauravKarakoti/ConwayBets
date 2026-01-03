import React, { useState, useEffect } from 'react';
import { useDynamicContext } from "@dynamic-labs/sdk-react-core"; //
import { WalletProvider } from './lib/wallet-connector';
import { getConwayBetsClient } from './lib/linera-client';
import { lineraAdapter } from './lib/linera-adapter'; // Import adapter directly
import { useMarkets } from './lib/hooks/useMarkets';
import { formatCurrency, formatAddress, timeUntil } from './lib/utils';
import './App.css';
import { WalletConnector } from './components/WalletConnector';

function App() {
  const [activeTab, setActiveTab] = useState<'markets' | 'portfolio' | 'create'>('markets');
  const [searchQuery, setSearchQuery] = useState('');
  const [showResolved, setShowResolved] = useState(false);
  
  // Get the wallet from Dynamic SDK
  const { primaryWallet } = useDynamicContext();
  const [isLineraReady, setIsLineraReady] = useState(false);

  // Initialize Linera Adapter when wallet connects
  useEffect(() => {
    const initLinera = async () => {
      if (!primaryWallet) {
        setIsLineraReady(false);
        return;
      }

      try {
        const rpcUrl = import.meta.env.VITE_LINERA_ENDPOINT || 'https://faucet.testnet-conway.linera.net';
        // Connect adapter with the dynamic wallet
        await lineraAdapter.connect(primaryWallet, rpcUrl);
        console.log("Linera adapter connected");
        
        // Connect client (sets application ID)
        const client = getConwayBetsClient();
        console.log("Connecting ConwayBets client...");
        await client.connect();
        
        setIsLineraReady(true);
        console.log("Linera initialization complete");
      } catch (error) {
        console.error("Failed to initialize Linera:", error);
      }
    };

    initLinera();
  }, [primaryWallet]);

  // Pass autoRefresh: isLineraReady to prevent fetching before ready
  const { markets, loading, error } = useMarkets({
    filter: {
      search: searchQuery,
      resolved: showResolved ? undefined : false,
    },
    enabled: isLineraReady,
    autoRefresh: isLineraReady, // Only refresh if ready
    refreshInterval: 15000,
  });

  const [newMarket, setNewMarket] = useState({
    title: '',
    description: '',
    endTime: '',
    outcomes: ['', ''],
  });

  const handleCreateMarket = async (e: React.FormEvent) => {
    e.preventDefault();
    console.log('Creating market:', newMarket);
  };

  const handlePlaceBet = async (marketId: string, outcomeIndex: number, amount: string) => {
    console.log('Placing bet:', { marketId, outcomeIndex, amount });
  };

  return (
    <WalletProvider>
      <div className="app">
        <header className="app-header">
          <div className="header-content">
            <div className="logo">
              <h1>ConwayBets</h1>
              <span className="subtitle">Real-time Prediction Markets</span>
            </div>
            
            <div className="header-actions">
              <div className="network-badge">
                <span className="network-dot"></span>
                Conway Testnet
              </div>
              <WalletConnector />
            </div>
          </div>

          <nav className="app-nav">
            <button
              className={`nav-button ${activeTab === 'markets' ? 'active' : ''}`}
              onClick={() => setActiveTab('markets')}
            >
              Markets
            </button>
            <button
              className={`nav-button ${activeTab === 'portfolio' ? 'active' : ''}`}
              onClick={() => setActiveTab('portfolio')}
            >
              Portfolio
            </button>
            <button
              className={`nav-button ${activeTab === 'create' ? 'active' : ''}`}
              onClick={() => setActiveTab('create')}
            >
              Create Market
            </button>
          </nav>
        </header>

        <main className="app-main">
          {!isLineraReady ? (
            // CHECK: If wallet is connected but Linera isn't ready, show loading
            primaryWallet ? (
              <div className="loading-container">
                <div className="spinner"></div>
                <p>Initializing Linera Client...</p>
              </div>
            ) : (
              // Show prompt only if no wallet is connected
              <div className="connect-prompt">
                <p>Please connect your wallet to view markets.</p>
              </div>
            )
          ) : (
            <>
              {activeTab === 'markets' && (
                <div className="markets-tab">
                  <div className="markets-header">
                    <div className="search-container">
                      <input
                        type="text"
                        placeholder="Search markets..."
                        value={searchQuery}
                        onChange={(e) => setSearchQuery(e.target.value)}
                        className="search-input"
                      />
                      <button className="search-button">🔍</button>
                    </div>

                    <div className="filters">
                      <label className="filter-toggle">
                        <input
                          type="checkbox"
                          checked={showResolved}
                          onChange={(e) => setShowResolved(e.target.checked)}
                        />
                        Show Resolved
                      </label>
                    </div>
                  </div>

                  {loading && markets.length === 0 ? (
                    <div className="loading-container"><div className="spinner"></div><p>Loading markets...</p></div>
                  ) : error ? (
                    <div className="error-container">
                      <p className="error-message">{error}</p>
                      <button onClick={() => window.location.reload()}>Retry</button>
                    </div>
                  ) : markets.length === 0 ? (
                    <div className="empty-state"><p>No markets found.</p></div>
                  ) : (
                    <div className="markets-grid">
                      {markets.map((market) => (
                        <div key={market.id} className="market-card">
                      <div className="market-header">
                            <h3 className="market-title">{market.title}</h3>
                            <span className={`market-status ${market.isResolved ? 'resolved' : 'active'}`}>
                              {market.isResolved ? 'Resolved' : timeUntil(market.endTime)}
                            </span>
                          </div>
                          <p className="market-description">{market.description}</p>
                          <div className="market-meta">
                            <span className="meta-item">Creator: {formatAddress(market.creator)}</span>
                            <span className="meta-item">Liquidity: {formatCurrency(market.totalLiquidity)} TEST</span>
                          </div>
                           <div className="market-outcomes">
                            {market.outcomes.map((outcome, index) => (
                              <div key={index} className="outcome-row">
                                <span className="outcome-label">{outcome}</span>
                                <div className="outcome-actions">
                                  <input type="number" placeholder="Amount" className="bet-input" min="1" step="1" />
                                  <button className="bet-button" onClick={() => handlePlaceBet(market.id, index, '10')}>Bet</button>
                                </div>
                              </div>
                            ))}
                          </div>
                          <div className="market-footer">
                            <button className="view-details-button">View Details</button>
                          </div>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              )}

              {activeTab === 'portfolio' && (
                <div className="portfolio-tab">
                  <h2>Your Portfolio</h2>
                  <div className="portfolio-stats">
                    <div className="stat-card">
                      <h3>Total Value</h3>
                      <p className="stat-value">0.00 TEST</p>
                    </div>
                    <div className="stat-card">
                      <h3>Active Bets</h3>
                      <p className="stat-value">0</p>
                    </div>
                    <div className="stat-card">
                      <h3>Total Profit</h3>
                      <p className="stat-value profit">+0.00 TEST</p>
                    </div>
                  </div>
                  
                  <div className="positions-list">
                    <h3>Your Positions</h3>
                    <div className="empty-positions">
                      <p>No active positions. Place your first bet!</p>
                    </div>
                  </div>
                </div>
              )}

              {activeTab === 'create' && (
                <div className="create-tab">
                  <h2>Create New Market</h2>
                  <form onSubmit={handleCreateMarket} className="create-market-form">
                    <div className="form-group">
                      <label htmlFor="market-title">Market Title</label>
                      <input id="market-title" type="text" value={newMarket.title} onChange={(e) => setNewMarket({...newMarket, title: e.target.value})} required />
                    </div>
                
                    <div className="form-group">
                      <label htmlFor="market-description">Description</label>
                      <textarea
                        id="market-description"
                        value={newMarket.description}
                        onChange={(e) => setNewMarket({...newMarket, description: e.target.value})}
                        placeholder="Describe the event and conditions..."
                        rows={4}
                        required
                      />
                    </div>
                    
                    <div className="form-group">
                      <label htmlFor="market-end">End Time</label>
                      <input
                        id="market-end"
                        type="datetime-local"
                        value={newMarket.endTime}
                        onChange={(e) => setNewMarket({...newMarket, endTime: e.target.value})}
                        required
                      />
                    </div>
                    
                    <div className="form-group">
                      <label>Outcomes</label>
                      {newMarket.outcomes.map((outcome, index) => (
                        <div key={index} className="outcome-input-row">
                          <input
                            type="text"
                            value={outcome}
                            onChange={(e) => {
                              const newOutcomes = [...newMarket.outcomes];
                              newOutcomes[index] = e.target.value;
                              setNewMarket({...newMarket, outcomes: newOutcomes});
                            }}
                            placeholder={`Outcome ${index + 1}`}
                            required
                          />
                          {index >= 2 && (
                            <button
                              type="button"
                              onClick={() => {
                                const newOutcomes = newMarket.outcomes.filter((_, i) => i !== index);
                                setNewMarket({...newMarket, outcomes: newOutcomes});
                              }}
                              className="remove-outcome-button"
                            >
                              Remove
                            </button>
                          )}
                        </div>
                      ))}
                      
                      {newMarket.outcomes.length < 4 && (
                        <button
                          type="button"
                          onClick={() => setNewMarket({
                            ...newMarket,
                            outcomes: [...newMarket.outcomes, '']
                          })}
                          className="add-outcome-button"
                        >
                          + Add Outcome
                        </button>
                      )}
                    </div>
                
                    <div className="form-actions">
                      <button type="submit" className="create-market-button">Create Market</button>
                    </div>
                  </form>
                </div>
              )}
            </>
          )}
        </main>

        <footer className="app-footer">
          <div className="footer-content">
            <div className="footer-section">
              <h4>ConwayBets</h4>
              <p>Real-time prediction markets on Linera Conway testnet</p>
            </div>
            
            <div className="footer-section">
              <h4>Links</h4>
              <a href="https://linera.io" target="_blank" rel="noopener noreferrer">
                Linera Network
              </a>
              <a href="https://linera.dev/protocol/overview.html" target="_blank" rel="noopener noreferrer">
                Documentation
              </a>
              <a href="https://github.com/GauravKarakoti/conwaybets" target="_blank" rel="noopener noreferrer">
                GitHub
              </a>
            </div>
            
            <div className="footer-section">
              <h4>Network Status</h4>
              <div className="status-indicator">
                <span className="status-dot online"></span>
                Conway Testnet Online
              </div>
              <p className="status-note">
                Average confirmation: &lt;1s
              </p>
            </div>
          </div>
          
          <div className="footer-bottom">
            <p>© 2024 ConwayBets. Built for Linera Buildathon.</p>
          </div>
        </footer>
      </div>
    </WalletProvider>
  );
}

export default App;
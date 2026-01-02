import { useState, useEffect } from 'react';

export const useMarketUpdates = (marketId: string) => {
    const [marketState, setMarketState] = useState(null);
    const endpoint = import.meta.env.VITE_LINERA_ENDPOINT || 'https://faucet.testnet-conway.linera.net';

    useEffect(() => {
        // Use Server-Sent Events (SSE) for updates
        // Note: Ensure your Linera service exposes this SSE endpoint
        const eventSource = new EventSource(`${endpoint}/events/market/${marketId}`);
        
        eventSource.onmessage = (event) => {
            try {
                setMarketState(JSON.parse(event.data));
            } catch (e) {
                console.error("Failed to parse market update", e);
            }
        };

        eventSource.onerror = (err) => {
            console.error("EventSource failed:", err);
            eventSource.close();
        };

        return () => {
            eventSource.close();
        };
    }, [marketId, endpoint]);

    return marketState;
};
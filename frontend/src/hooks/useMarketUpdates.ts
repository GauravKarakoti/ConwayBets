import { useState, useEffect } from 'react';
import { LineraClient } from '@linera/client';

export const useMarketUpdates = (marketId: string) => {
    const [marketState, setMarketState] = useState(null);
    const client = new LineraClient({ endpoint: process.env.LINERA_ENDPOINT });

    useEffect(() => {
        // Primary: GraphQL Subscription
        const subscription = client.subscribe(
            `subscription { marketState(id: "${marketId}") { id, totalLiquidity, isResolved } }`,
            { marketId }
        ).subscribe(({ data }: any) => {
            setMarketState(data.marketState);
        });

        // Fallback: Server-Sent Events (SSE)
        const eventSource = new EventSource(`https://conway.linera.io/events/market/${marketId}`);
        eventSource.onmessage = (event) => {
            setMarketState(JSON.parse(event.data));
        };

        return () => {
            subscription.unsubscribe();
            eventSource.close();
        };
    }, [marketId]);

    return marketState;
};
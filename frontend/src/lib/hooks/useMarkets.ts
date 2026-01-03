import { useState, useEffect, useCallback, useMemo } from 'react';
import { type Market, getConwayBetsClient } from '../linera-client';
import { lineraAdapter } from '../linera-adapter';
import { debounce } from '../utils';

export interface UseMarketsOptions {
  limit?: number;
  offset?: number;
  filter?: {
    resolved?: boolean;
    creator?: string;
    search?: string;
  };
  autoRefresh?: boolean;
  refreshInterval?: number;
  enabled?: boolean;
}

export function useMarkets(options: UseMarketsOptions = {}) {
  const {
    limit = 20,
    filter = {},
    autoRefresh = false,
    refreshInterval = 10000,
    enabled = true,
  } = options;

  const [markets, setMarkets] = useState<Market[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [hasMore, setHasMore] = useState(true);
  const [total, setTotal] = useState(0);

  const client = getConwayBetsClient();

  // FIX: Stabilize the filter object.
  // The 'filter' passed from App.tsx is a new object reference on every render.
  // We use JSON.stringify to ensure we only update when the CONTENT changes.
  const stableFilter = useMemo(() => filter, [JSON.stringify(filter)]);

  const loadMarkets = useCallback(
    debounce(async (offset: number) => {
      if (!lineraAdapter.isApplicationSet()) {
        setLoading(false);
        return;
      }

      try {
        setLoading(true);
        setError(null);

        const loadedMarkets = await client.getAllMarkets(limit, offset);

        // Apply filters using the stableFilter captured in closure
        let filteredMarkets = loadedMarkets;
        
        if (stableFilter.resolved !== undefined) {
          filteredMarkets = filteredMarkets.filter(
            market => market.isResolved === stableFilter.resolved
          );
        }
        
        if (stableFilter.creator) {
          filteredMarkets = filteredMarkets.filter(
            market => market.creator.toLowerCase() === stableFilter.creator!.toLowerCase()
          );
        }
        
        if (stableFilter.search) {
          const searchLower = stableFilter.search.toLowerCase();
          filteredMarkets = filteredMarkets.filter(
            market =>
              market.title.toLowerCase().includes(searchLower) ||
              market.description.toLowerCase().includes(searchLower)
          );
        }

        const isLoadMore = offset > 0;

        setMarkets(prev => 
          isLoadMore ? [...prev, ...filteredMarkets] : filteredMarkets
        );
        setHasMore(filteredMarkets.length === limit);
        setTotal(prev => isLoadMore ? prev + filteredMarkets.length : filteredMarkets.length);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load markets');
        console.error('Failed to load markets:', err);
      } finally {
        setLoading(false);
      }
    }, 300),
    [client, limit, stableFilter] // Use stableFilter in dependency
  );

  const refresh = useCallback(() => {
    loadMarkets(0);
  }, [loadMarkets]);

  const loadMore = useCallback(() => {
    if (!loading && hasMore) {
      loadMarkets(markets.length);
    }
  }, [loading, hasMore, loadMarkets, markets.length]);

  useEffect(() => {
    if (enabled) {
      loadMarkets(0);
    }
  }, [stableFilter, loadMarkets, autoRefresh, enabled]); // Use stableFilter here too

  useEffect(() => {
    if (!autoRefresh) return;

    const interval = setInterval(() => {
      if (!loading) {
        loadMarkets(0);
      }
    }, refreshInterval);

    return () => clearInterval(interval);
  }, [autoRefresh, refreshInterval, loading, loadMarkets]);

  return {
    markets,
    loading,
    error,
    hasMore,
    total,
    refresh,
    loadMore,
  };
}
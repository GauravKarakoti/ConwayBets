import { useState, useEffect, useCallback } from 'react';
import { type Market, getConwayBetsClient } from '../linera-client';
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
}

export function useMarkets(options: UseMarketsOptions = {}) {
  const {
    limit = 20,
    offset = 0,
    filter = {},
    autoRefresh = false,
    refreshInterval = 10000,
  } = options;

  const [markets, setMarkets] = useState<Market[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [hasMore, setHasMore] = useState(true);
  const [total, setTotal] = useState(0);

  const client = getConwayBetsClient();

  const loadMarkets = useCallback(
    debounce(async (loadMore: boolean = false) => {
      try {
        setLoading(true);
        setError(null);

        const currentOffset = loadMore ? markets.length : 0;
        const loadedMarkets = await client.getAllMarkets(limit, currentOffset);

        // Apply filters
        let filteredMarkets = loadedMarkets;
        
        if (filter.resolved !== undefined) {
          filteredMarkets = filteredMarkets.filter(
            market => market.isResolved === filter.resolved
          );
        }
        
        if (filter.creator) {
          filteredMarkets = filteredMarkets.filter(
            market => market.creator.toLowerCase() === filter.creator!.toLowerCase()
          );
        }
        
        if (filter.search) {
          const searchLower = filter.search.toLowerCase();
          filteredMarkets = filteredMarkets.filter(
            market =>
              market.title.toLowerCase().includes(searchLower) ||
              market.description.toLowerCase().includes(searchLower)
          );
        }

        setMarkets(prev => 
          loadMore ? [...prev, ...filteredMarkets] : filteredMarkets
        );
        setHasMore(filteredMarkets.length === limit);
        setTotal(prev => loadMore ? prev + filteredMarkets.length : filteredMarkets.length);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load markets');
        console.error('Failed to load markets:', err);
      } finally {
        setLoading(false);
      }
    }, 300),
    [client, limit, filter, markets.length]
  );

  const refresh = useCallback(() => {
    loadMarkets(false);
  }, [loadMarkets]);

  const loadMore = useCallback(() => {
    if (!loading && hasMore) {
      loadMarkets(true);
    }
  }, [loading, hasMore, loadMarkets]);

  useEffect(() => {
    loadMarkets(false);
  }, [filter, loadMarkets]);

  useEffect(() => {
    if (!autoRefresh) return;

    const interval = setInterval(() => {
      if (!loading) {
        loadMarkets(false);
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
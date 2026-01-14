<template>
  <div class="dashboard">
    <h1 class="page-title">Dashboard</h1>
    
    <!-- Stats Cards -->
    <div class="stats-grid">
      <div class="stat-card">
        <div class="stat-icon">📊</div>
        <div class="stat-content">
          <span class="stat-label">Total P&L</span>
          <span class="stat-value" :class="{ positive: totalPnl >= 0, negative: totalPnl < 0 }">
            {{ totalPnl >= 0 ? '+' : '' }}${{ totalPnl.toFixed(2) }}
          </span>
        </div>
      </div>
      <div class="stat-card">
        <div class="stat-icon">📈</div>
        <div class="stat-content">
          <span class="stat-label">Active Copies</span>
          <span class="stat-value">{{ copyRelations.length }}</span>
        </div>
      </div>
      <div class="stat-card">
        <div class="stat-icon">💹</div>
        <div class="stat-content">
          <span class="stat-label">Open Positions</span>
          <span class="stat-value">{{ openTrades.length }}</span>
        </div>
      </div>
      <div class="stat-card">
        <div class="stat-icon">🏆</div>
        <div class="stat-content">
          <span class="stat-label">Win Rate</span>
          <span class="stat-value">{{ (winRate * 100).toFixed(1) }}%</span>
        </div>
      </div>
    </div>

    <!-- Open Trades -->
    <section class="section">
      <h2 class="section-title">Open Trades</h2>
      <div v-if="loadingTrades" class="loading">Loading trades...</div>
      <div v-else-if="openTrades.length === 0" class="empty-state">
        No open trades. Start by copying a trader!
      </div>
      <div v-else class="trades-grid">
        <div v-for="trade in openTrades" :key="trade.id" class="trade-card">
          <div class="trade-header">
            <span class="trade-symbol">{{ trade.symbol }}</span>
            <span class="trade-direction" :class="trade.direction.toLowerCase()">
              {{ trade.direction }}
            </span>
          </div>
          <div class="trade-details">
            <div class="detail-row">
              <span class="detail-label">Entry Price</span>
              <span class="detail-value">${{ trade.entryPrice.toLocaleString() }}</span>
            </div>
            <div class="detail-row">
              <span class="detail-label">Quantity</span>
              <span class="detail-value">{{ trade.quantity }}</span>
            </div>
            <div class="detail-row">
              <span class="detail-label">Value</span>
              <span class="detail-value">${{ (trade.entryPrice * trade.quantity).toLocaleString() }}</span>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- Top Traders Preview -->
    <section class="section">
      <div class="section-header">
        <h2 class="section-title">Top Traders</h2>
        <router-link to="/traders" class="view-all">View All →</router-link>
      </div>
      <div v-if="loadingTraders" class="loading">Loading traders...</div>
      <div v-else class="traders-preview">
        <div v-for="trader in topTraders" :key="trader.id" class="trader-preview-card">
          <div class="trader-avatar">{{ trader.username.charAt(0) }}</div>
          <div class="trader-info">
            <span class="trader-name">{{ trader.username }}</span>
            <span class="trader-stats">
              {{ trader.followersCount }} followers • {{ (trader.winRate * 100).toFixed(0) }}% win
            </span>
          </div>
          <div class="trader-pnl" :class="{ positive: trader.totalPnl >= 0 }">
            +${{ trader.totalPnl.toLocaleString() }}
          </div>
        </div>
      </div>
    </section>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'
import { useQuery } from '@vue/apollo-composable'
import { GET_TRADERS, GET_OPEN_TRADES, GET_MY_COPY_RELATIONS, GET_MY_COPIED_TRADES } from '../graphql/queries'

const currentUserId = 'user1'

// Queries
const { result: tradersResult, loading: loadingTraders } = useQuery(GET_TRADERS)
const { result: openTradesResult, loading: loadingTrades } = useQuery(GET_OPEN_TRADES)
const { result: copyRelationsResult } = useQuery(GET_MY_COPY_RELATIONS, { followerId: currentUserId })
const { result: copiedTradesResult } = useQuery(GET_MY_COPIED_TRADES, { followerId: currentUserId })

// Computed
const topTraders = computed(() => {
  const traders = tradersResult.value?.traders || []
  return [...traders].sort((a, b) => b.totalPnl - a.totalPnl).slice(0, 3)
})

const openTrades = computed(() => openTradesResult.value?.openTrades || [])
const copyRelations = computed(() => copyRelationsResult.value?.myCopyRelations || [])
const copiedTrades = computed(() => copiedTradesResult.value?.myCopiedTrades || [])

const totalPnl = computed(() => {
  return copiedTrades.value.reduce((sum, trade) => sum + (trade.pnl || 0), 0)
})

const winRate = computed(() => {
  const closedTrades = copiedTrades.value.filter(t => t.status === 'CLOSED')
  if (closedTrades.length === 0) return 0.65
  const wins = closedTrades.filter(t => (t.pnl || 0) > 0).length
  return wins / closedTrades.length
})
</script>

<style scoped>
.dashboard {
  animation: fadeIn 0.3s ease;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}

.page-title {
  font-size: 1.75rem;
  font-weight: 700;
  margin-bottom: 1.5rem;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 1rem;
  margin-bottom: 2rem;
}

.stat-card {
  background: #16202a;
  border-radius: 12px;
  padding: 1.25rem;
  display: flex;
  align-items: center;
  gap: 1rem;
  border: 1px solid #2f3336;
}

.stat-icon {
  font-size: 2rem;
  background: #1d2d3a;
  padding: 0.75rem;
  border-radius: 10px;
}

.stat-content {
  display: flex;
  flex-direction: column;
}

.stat-label {
  color: #8899a6;
  font-size: 0.85rem;
}

.stat-value {
  font-size: 1.5rem;
  font-weight: 700;
}

.stat-value.positive { color: #00d4aa; }
.stat-value.negative { color: #f4212e; }

.section {
  margin-bottom: 2rem;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1rem;
}

.section-title {
  font-size: 1.25rem;
  font-weight: 600;
  margin-bottom: 1rem;
}

.view-all {
  color: #00a3ff;
  text-decoration: none;
  font-size: 0.9rem;
}

.loading, .empty-state {
  color: #8899a6;
  padding: 2rem;
  text-align: center;
  background: #16202a;
  border-radius: 12px;
  border: 1px solid #2f3336;
}

.trades-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 1rem;
}

.trade-card {
  background: #16202a;
  border-radius: 12px;
  padding: 1.25rem;
  border: 1px solid #2f3336;
}

.trade-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1rem;
  padding-bottom: 0.75rem;
  border-bottom: 1px solid #2f3336;
}

.trade-symbol {
  font-weight: 700;
  font-size: 1.1rem;
}

.trade-direction {
  padding: 0.25rem 0.75rem;
  border-radius: 6px;
  font-size: 0.8rem;
  font-weight: 600;
  text-transform: uppercase;
}

.trade-direction.long {
  background: #00d4aa22;
  color: #00d4aa;
}

.trade-direction.short {
  background: #f4212e22;
  color: #f4212e;
}

.trade-details {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.detail-row {
  display: flex;
  justify-content: space-between;
}

.detail-label {
  color: #8899a6;
  font-size: 0.9rem;
}

.detail-value {
  font-weight: 500;
}

.traders-preview {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.trader-preview-card {
  display: flex;
  align-items: center;
  gap: 1rem;
  background: #16202a;
  padding: 1rem 1.25rem;
  border-radius: 12px;
  border: 1px solid #2f3336;
}

.trader-avatar {
  width: 48px;
  height: 48px;
  background: linear-gradient(135deg, #00d4aa 0%, #00a3ff 100%);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 700;
  font-size: 1.25rem;
}

.trader-info {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.trader-name {
  font-weight: 600;
}

.trader-stats {
  color: #8899a6;
  font-size: 0.85rem;
}

.trader-pnl {
  font-weight: 700;
  font-size: 1.1rem;
}

.trader-pnl.positive {
  color: #00d4aa;
}
</style>

<template>
  <div class="my-trades-page">
    <h1 class="page-title">My Trades</h1>
    <p class="page-subtitle">Track your copied positions and performance</p>

    <!-- Summary Stats -->
    <div class="summary-stats">
      <div class="summary-card">
        <span class="summary-label">Open Positions</span>
        <span class="summary-value">{{ openPositions.length }}</span>
      </div>
      <div class="summary-card">
        <span class="summary-label">Closed Trades</span>
        <span class="summary-value">{{ closedPositions.length }}</span>
      </div>
      <div class="summary-card">
        <span class="summary-label">Total Realized P&L</span>
        <span class="summary-value" :class="{ positive: totalRealizedPnl >= 0, negative: totalRealizedPnl < 0 }">
          {{ totalRealizedPnl >= 0 ? '+' : '' }}${{ totalRealizedPnl.toFixed(2) }}
        </span>
      </div>
    </div>

    <!-- Tabs -->
    <div class="tabs">
      <button 
        class="tab" 
        :class="{ active: activeTab === 'open' }" 
        @click="activeTab = 'open'"
      >
        Open ({{ openPositions.length }})
      </button>
      <button 
        class="tab" 
        :class="{ active: activeTab === 'closed' }" 
        @click="activeTab = 'closed'"
      >
        Closed ({{ closedPositions.length }})
      </button>
      <button 
        class="tab" 
        :class="{ active: activeTab === 'following' }" 
        @click="activeTab = 'following'"
      >
        Following ({{ copyRelations.length }})
      </button>
    </div>

    <!-- Open Positions Tab -->
    <div v-if="activeTab === 'open'" class="tab-content">
      <div v-if="loading" class="loading">Loading positions...</div>
      <div v-else-if="openPositions.length === 0" class="empty-state">
        <div class="empty-icon">📭</div>
        <p>No open positions</p>
        <router-link to="/traders" class="cta-link">Find traders to copy →</router-link>
      </div>
      <div v-else class="positions-table">
        <div class="table-header">
          <span>Trade</span>
          <span>Direction</span>
          <span>Entry</span>
          <span>Quantity</span>
          <span>Value</span>
          <span>Status</span>
        </div>
        <div v-for="position in openPositions" :key="position.id" class="table-row">
          <span class="trade-info">
            <span class="symbol">{{ getOriginalTrade(position.originalTradeId)?.symbol || 'N/A' }}</span>
          </span>
          <span class="direction" :class="getOriginalTrade(position.originalTradeId)?.direction?.toLowerCase()">
            {{ getOriginalTrade(position.originalTradeId)?.direction || '-' }}
          </span>
          <span>${{ getOriginalTrade(position.originalTradeId)?.entryPrice?.toLocaleString() || '-' }}</span>
          <span>{{ position.quantity.toFixed(4) }}</span>
          <span>${{ ((getOriginalTrade(position.originalTradeId)?.entryPrice || 0) * position.quantity).toFixed(2) }}</span>
          <span class="status open">Open</span>
        </div>
      </div>
    </div>

    <!-- Closed Positions Tab -->
    <div v-if="activeTab === 'closed'" class="tab-content">
      <div v-if="closedPositions.length === 0" class="empty-state">
        <div class="empty-icon">📊</div>
        <p>No closed trades yet</p>
      </div>
      <div v-else class="positions-table">
        <div class="table-header">
          <span>Trade</span>
          <span>Direction</span>
          <span>Entry</span>
          <span>Exit</span>
          <span>Quantity</span>
          <span>P&L</span>
        </div>
        <div v-for="position in closedPositions" :key="position.id" class="table-row">
          <span class="trade-info">
            <span class="symbol">{{ getOriginalTrade(position.originalTradeId)?.symbol || 'N/A' }}</span>
          </span>
          <span class="direction" :class="getOriginalTrade(position.originalTradeId)?.direction?.toLowerCase()">
            {{ getOriginalTrade(position.originalTradeId)?.direction || '-' }}
          </span>
          <span>${{ getOriginalTrade(position.originalTradeId)?.entryPrice?.toLocaleString() || '-' }}</span>
          <span>${{ getOriginalTrade(position.originalTradeId)?.exitPrice?.toLocaleString() || '-' }}</span>
          <span>{{ position.quantity.toFixed(4) }}</span>
          <span class="pnl" :class="{ positive: (position.pnl || 0) >= 0, negative: (position.pnl || 0) < 0 }">
            {{ position.pnl >= 0 ? '+' : '' }}${{ (position.pnl || 0).toFixed(2) }}
          </span>
        </div>
      </div>
    </div>

    <!-- Following Tab -->
    <div v-if="activeTab === 'following'" class="tab-content">
      <div v-if="copyRelations.length === 0" class="empty-state">
        <div class="empty-icon">👥</div>
        <p>Not following any traders</p>
        <router-link to="/traders" class="cta-link">Browse traders →</router-link>
      </div>
      <div v-else class="following-list">
        <div v-for="relation in copyRelations" :key="relation.id" class="following-card">
          <div class="following-info">
            <div class="trader-avatar">{{ getTrader(relation.traderId)?.username?.charAt(0) || '?' }}</div>
            <div class="trader-details">
              <span class="trader-name">{{ getTrader(relation.traderId)?.username || 'Unknown' }}</span>
              <span class="copy-ratio">Copying {{ (relation.copyRatio * 100).toFixed(0) }}% of trades</span>
            </div>
          </div>
          <div class="following-stats">
            <div class="stat">
              <span class="stat-value positive">+${{ getTrader(relation.traderId)?.totalPnl?.toLocaleString() || '0' }}</span>
              <span class="stat-label">Their P&L</span>
            </div>
            <div class="stat">
              <span class="stat-value">{{ (getTrader(relation.traderId)?.winRate * 100 || 0).toFixed(0) }}%</span>
              <span class="stat-label">Win Rate</span>
            </div>
          </div>
          <button @click="handleStopCopying(relation.id)" class="stop-btn">Stop Copying</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'
import { useQuery, useMutation } from '@vue/apollo-composable'
import { 
  GET_MY_COPIED_TRADES, 
  GET_MY_COPY_RELATIONS, 
  GET_TRADES, 
  GET_TRADERS,
  STOP_COPYING 
} from '../graphql/queries'

const currentUserId = 'user1'
const activeTab = ref('open')

// Queries
const { result: copiedTradesResult, loading } = useQuery(GET_MY_COPIED_TRADES, { followerId: currentUserId })
const { result: relationsResult, refetch: refetchRelations } = useQuery(GET_MY_COPY_RELATIONS, { followerId: currentUserId })
const { result: tradesResult } = useQuery(GET_TRADES, { traderId: null })
const { result: tradersResult } = useQuery(GET_TRADERS)

// Mutations
const { mutate: stopCopyingMutation } = useMutation(STOP_COPYING)

// Computed
const copiedTrades = computed(() => copiedTradesResult.value?.myCopiedTrades || [])
const copyRelations = computed(() => relationsResult.value?.myCopyRelations || [])
const allTrades = computed(() => tradesResult.value?.trades || [])
const traders = computed(() => tradersResult.value?.traders || [])

const openPositions = computed(() => copiedTrades.value.filter(t => t.status === 'OPEN'))
const closedPositions = computed(() => copiedTrades.value.filter(t => t.status === 'CLOSED'))

const totalRealizedPnl = computed(() => {
  return closedPositions.value.reduce((sum, t) => sum + (t.pnl || 0), 0)
})

// Methods
const getOriginalTrade = (tradeId) => {
  return allTrades.value.find(t => t.id === tradeId)
}

const getTrader = (traderId) => {
  return traders.value.find(t => t.id === traderId)
}

const handleStopCopying = async (relationId) => {
  try {
    await stopCopyingMutation({ relationId })
    await refetchRelations()
  } catch (error) {
    console.error('Failed to stop copying:', error)
  }
}
</script>

<style scoped>
.my-trades-page {
  animation: fadeIn 0.3s ease;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(10px); }
  to { opacity: 1; transform: translateY(0); }
}

.page-title {
  font-size: 1.75rem;
  font-weight: 700;
  margin-bottom: 0.5rem;
}

.page-subtitle {
  color: #8899a6;
  margin-bottom: 2rem;
}

.summary-stats {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 1rem;
  margin-bottom: 2rem;
}

.summary-card {
  background: #16202a;
  padding: 1.25rem;
  border-radius: 12px;
  border: 1px solid #2f3336;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.summary-label {
  color: #8899a6;
  font-size: 0.85rem;
}

.summary-value {
  font-size: 1.5rem;
  font-weight: 700;
}

.summary-value.positive { color: #00d4aa; }
.summary-value.negative { color: #f4212e; }

.tabs {
  display: flex;
  gap: 0.5rem;
  margin-bottom: 1.5rem;
  border-bottom: 1px solid #2f3336;
  padding-bottom: 0.5rem;
}

.tab {
  background: transparent;
  border: none;
  color: #8899a6;
  padding: 0.75rem 1.25rem;
  cursor: pointer;
  font-size: 0.95rem;
  border-radius: 8px;
  transition: all 0.2s;
}

.tab:hover {
  background: #1d2d3a;
  color: #e7e9ea;
}

.tab.active {
  background: #1d2d3a;
  color: #00d4aa;
  font-weight: 600;
}

.tab-content {
  min-height: 300px;
}

.loading, .empty-state {
  background: #16202a;
  border-radius: 12px;
  padding: 3rem;
  text-align: center;
  color: #8899a6;
  border: 1px solid #2f3336;
}

.empty-icon {
  font-size: 3rem;
  margin-bottom: 1rem;
}

.cta-link {
  display: inline-block;
  margin-top: 1rem;
  color: #00a3ff;
  text-decoration: none;
}

.positions-table {
  background: #16202a;
  border-radius: 12px;
  border: 1px solid #2f3336;
  overflow: hidden;
}

.table-header {
  display: grid;
  grid-template-columns: 1.5fr 1fr 1fr 1fr 1fr 1fr;
  padding: 1rem 1.25rem;
  background: #1d2d3a;
  font-weight: 600;
  font-size: 0.85rem;
  color: #8899a6;
  text-transform: uppercase;
}

.table-row {
  display: grid;
  grid-template-columns: 1.5fr 1fr 1fr 1fr 1fr 1fr;
  padding: 1rem 1.25rem;
  border-top: 1px solid #2f3336;
  align-items: center;
}

.table-row:hover {
  background: #1d2d3a44;
}

.symbol {
  font-weight: 600;
}

.direction {
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
  font-size: 0.75rem;
  font-weight: 600;
  text-transform: uppercase;
  width: fit-content;
}

.direction.long {
  background: #00d4aa22;
  color: #00d4aa;
}

.direction.short {
  background: #f4212e22;
  color: #f4212e;
}

.status {
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
  font-size: 0.75rem;
  font-weight: 600;
  width: fit-content;
}

.status.open {
  background: #00a3ff22;
  color: #00a3ff;
}

.pnl {
  font-weight: 700;
}

.pnl.positive { color: #00d4aa; }
.pnl.negative { color: #f4212e; }

.following-list {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.following-card {
  background: #16202a;
  border-radius: 12px;
  padding: 1.25rem;
  border: 1px solid #2f3336;
  display: flex;
  align-items: center;
  gap: 1.5rem;
}

.following-info {
  display: flex;
  align-items: center;
  gap: 1rem;
  flex: 1;
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

.trader-details {
  display: flex;
  flex-direction: column;
}

.trader-name {
  font-weight: 600;
  font-size: 1.1rem;
}

.copy-ratio {
  color: #8899a6;
  font-size: 0.85rem;
}

.following-stats {
  display: flex;
  gap: 2rem;
}

.stat {
  text-align: center;
}

.stat-value {
  display: block;
  font-size: 1.1rem;
  font-weight: 700;
}

.stat-value.positive {
  color: #00d4aa;
}

.stat-label {
  color: #8899a6;
  font-size: 0.75rem;
}

.stop-btn {
  background: #f4212e22;
  color: #f4212e;
  border: none;
  padding: 0.6rem 1.25rem;
  border-radius: 8px;
  cursor: pointer;
  font-weight: 500;
  transition: background 0.2s;
}

.stop-btn:hover {
  background: #f4212e33;
}
</style>

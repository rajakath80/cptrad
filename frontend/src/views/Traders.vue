<template>
  <div class="traders-page">
    <h1 class="page-title">Top Traders</h1>
    <p class="page-subtitle">Find and copy the best performing traders</p>

    <div v-if="loading" class="loading">Loading traders...</div>
    
    <div v-else class="traders-list">
      <div v-for="trader in traders" :key="trader.id" class="trader-card">
        <div class="trader-header">
          <div class="trader-avatar">{{ trader.username.charAt(0) }}</div>
          <div class="trader-main-info">
            <h3 class="trader-name">{{ trader.username }}</h3>
            <span class="trader-badge">Pro Trader</span>
          </div>
        </div>

        <div class="trader-stats">
          <div class="stat">
            <span class="stat-value positive">+${{ trader.totalPnl.toLocaleString() }}</span>
            <span class="stat-label">Total P&L</span>
          </div>
          <div class="stat">
            <span class="stat-value">{{ (trader.winRate * 100).toFixed(0) }}%</span>
            <span class="stat-label">Win Rate</span>
          </div>
          <div class="stat">
            <span class="stat-value">{{ trader.followersCount }}</span>
            <span class="stat-label">Followers</span>
          </div>
        </div>

        <div class="copy-section">
          <div v-if="!isFollowing(trader.id)" class="copy-controls">
            <div class="ratio-selector">
              <label>Copy Ratio:</label>
              <select v-model="copyRatios[trader.id]" class="ratio-select">
                <option value="0.1">10%</option>
                <option value="0.25">25%</option>
                <option value="0.5">50%</option>
                <option value="1">100%</option>
              </select>
            </div>
            <button @click="startCopying(trader.id)" class="copy-btn" :disabled="copying">
              {{ copying ? 'Starting...' : 'Start Copying' }}
            </button>
          </div>
          <div v-else class="following-badge">
            <span>✓ Following</span>
            <button @click="stopCopyingTrader(trader.id)" class="stop-btn">Stop</button>
          </div>
        </div>

        <!-- Recent Trades Preview -->
        <div class="recent-trades">
          <h4>Recent Trades</h4>
          <div class="trades-mini-list">
            <div v-for="trade in getTraderTrades(trader.id).slice(0, 3)" :key="trade.id" class="trade-mini">
              <span class="trade-mini-symbol">{{ trade.symbol }}</span>
              <span class="trade-mini-direction" :class="trade.direction.toLowerCase()">
                {{ trade.direction }}
              </span>
              <span class="trade-mini-pnl" :class="{ positive: (trade.pnl || 0) >= 0, negative: (trade.pnl || 0) < 0 }">
                {{ trade.pnl ? `${trade.pnl >= 0 ? '+' : ''}$${trade.pnl.toFixed(2)}` : 'Open' }}
              </span>
            </div>
            <div v-if="getTraderTrades(trader.id).length === 0" class="no-trades">
              No recent trades
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Success Toast -->
    <div v-if="showToast" class="toast" :class="toastType">
      {{ toastMessage }}
    </div>
  </div>
</template>

<script setup>
import { ref, computed, reactive } from 'vue'
import { useQuery, useMutation } from '@vue/apollo-composable'
import { GET_TRADERS, GET_TRADES, GET_MY_COPY_RELATIONS, COPY_TRADER, STOP_COPYING } from '../graphql/queries'

const currentUserId = 'user1'

// State
const copyRatios = reactive({})
const copying = ref(false)
const showToast = ref(false)
const toastMessage = ref('')
const toastType = ref('success')

// Queries
const { result: tradersResult, loading } = useQuery(GET_TRADERS)
const { result: tradesResult } = useQuery(GET_TRADES, { traderId: null })
const { result: relationsResult, refetch: refetchRelations } = useQuery(GET_MY_COPY_RELATIONS, { followerId: currentUserId })

// Mutations
const { mutate: copyTraderMutation } = useMutation(COPY_TRADER)
const { mutate: stopCopyingMutation } = useMutation(STOP_COPYING)

// Computed
const traders = computed(() => {
  const tradersList = tradersResult.value?.traders || []
  // Initialize copy ratios
  tradersList.forEach(t => {
    if (!copyRatios[t.id]) copyRatios[t.id] = '0.25'
  })
  return [...tradersList].sort(((a, b) => b.totalPnl - a.totalPnl))
})

const allTrades = computed(() => tradesResult.value?.trades || [])
const copyRelations = computed(() => relationsResult.value?.myCopyRelations || [])

// Methods
const getTraderTrades = (traderId) => {
  return allTrades.value.filter(t => t.traderId === traderId)
}

const isFollowing = (traderId) => {
  return copyRelations.value.some(r => r.traderId === traderId && r.active)
}

const getRelationId = (traderId) => {
  const relation = copyRelations.value.find(r => r.traderId === traderId && r.active)
  return relation?.id
}

const showToastMessage = (message, type = 'success') => {
  toastMessage.value = message
  toastType.value = type
  showToast.value = true
  setTimeout(() => { showToast.value = false }, 3000)
}

const startCopying = async (traderId) => {
  copying.value = true
  try {
    await copyTraderMutation({
      input: {
        followerId: currentUserId,
        traderId: traderId,
        copyRatio: parseFloat(copyRatios[traderId])
      }
    })
    await refetchRelations()
    showToastMessage('Successfully started copying trader!')
  } catch (error) {
    showToastMessage('Failed to start copying', 'error')
    console.error(error)
  }
  copying.value = false
}

const stopCopyingTrader = async (traderId) => {
  const relationId = getRelationId(traderId)
  if (!relationId) return
  
  try {
    await stopCopyingMutation({ relationId })
    await refetchRelations()
    showToastMessage('Stopped copying trader')
  } catch (error) {
    showToastMessage('Failed to stop copying', 'error')
    console.error(error)
  }
}
</script>

<style scoped>
.traders-page {
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

.loading {
  color: #8899a6;
  padding: 3rem;
  text-align: center;
  background: #16202a;
  border-radius: 12px;
}

.traders-list {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(380px, 1fr));
  gap: 1.5rem;
}

.trader-card {
  background: #16202a;
  border-radius: 16px;
  padding: 1.5rem;
  border: 1px solid #2f3336;
  transition: transform 0.2s, box-shadow 0.2s;
}

.trader-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 30px rgba(0, 0, 0, 0.3);
}

.trader-header {
  display: flex;
  align-items: center;
  gap: 1rem;
  margin-bottom: 1.25rem;
}

.trader-avatar {
  width: 56px;
  height: 56px;
  background: linear-gradient(135deg, #00d4aa 0%, #00a3ff 100%);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 700;
  font-size: 1.5rem;
}

.trader-main-info {
  flex: 1;
}

.trader-name {
  font-size: 1.25rem;
  font-weight: 700;
  margin-bottom: 0.25rem;
}

.trader-badge {
  background: linear-gradient(135deg, #ffd700 0%, #ffaa00 100%);
  color: #000;
  padding: 0.2rem 0.6rem;
  border-radius: 4px;
  font-size: 0.7rem;
  font-weight: 600;
  text-transform: uppercase;
}

.trader-stats {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 1rem;
  padding: 1rem 0;
  border-top: 1px solid #2f3336;
  border-bottom: 1px solid #2f3336;
  margin-bottom: 1rem;
}

.stat {
  text-align: center;
}

.stat-value {
  display: block;
  font-size: 1.25rem;
  font-weight: 700;
}

.stat-value.positive {
  color: #00d4aa;
}

.stat-label {
  color: #8899a6;
  font-size: 0.8rem;
}

.copy-section {
  margin-bottom: 1rem;
}

.copy-controls {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.ratio-selector {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  color: #8899a6;
  font-size: 0.9rem;
}

.ratio-select {
  background: #1d2d3a;
  border: 1px solid #2f3336;
  color: #e7e9ea;
  padding: 0.5rem 0.75rem;
  border-radius: 8px;
  cursor: pointer;
}

.copy-btn {
  flex: 1;
  background: linear-gradient(135deg, #00d4aa 0%, #00a3ff 100%);
  color: #fff;
  border: none;
  padding: 0.75rem 1.5rem;
  border-radius: 8px;
  font-weight: 600;
  cursor: pointer;
  transition: opacity 0.2s, transform 0.2s;
}

.copy-btn:hover:not(:disabled) {
  opacity: 0.9;
  transform: scale(1.02);
}

.copy-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.following-badge {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: #00d4aa22;
  color: #00d4aa;
  padding: 0.75rem 1rem;
  border-radius: 8px;
  font-weight: 600;
}

.stop-btn {
  background: #f4212e22;
  color: #f4212e;
  border: none;
  padding: 0.4rem 0.8rem;
  border-radius: 6px;
  cursor: pointer;
  font-weight: 500;
}

.stop-btn:hover {
  background: #f4212e33;
}

.recent-trades h4 {
  font-size: 0.9rem;
  color: #8899a6;
  margin-bottom: 0.75rem;
}

.trades-mini-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.trade-mini {
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: #1d2d3a;
  padding: 0.5rem 0.75rem;
  border-radius: 6px;
  font-size: 0.85rem;
}

.trade-mini-symbol {
  font-weight: 600;
}

.trade-mini-direction {
  padding: 0.15rem 0.5rem;
  border-radius: 4px;
  font-size: 0.7rem;
  text-transform: uppercase;
}

.trade-mini-direction.long {
  background: #00d4aa22;
  color: #00d4aa;
}

.trade-mini-direction.short {
  background: #f4212e22;
  color: #f4212e;
}

.trade-mini-pnl {
  font-weight: 600;
}

.trade-mini-pnl.positive { color: #00d4aa; }
.trade-mini-pnl.negative { color: #f4212e; }

.no-trades {
  color: #8899a6;
  font-size: 0.85rem;
  text-align: center;
  padding: 0.5rem;
}

.toast {
  position: fixed;
  bottom: 2rem;
  right: 2rem;
  padding: 1rem 1.5rem;
  border-radius: 8px;
  font-weight: 500;
  animation: slideIn 0.3s ease;
  z-index: 1000;
}

.toast.success {
  background: #00d4aa;
  color: #000;
}

.toast.error {
  background: #f4212e;
  color: #fff;
}

@keyframes slideIn {
  from { transform: translateX(100%); opacity: 0; }
  to { transform: translateX(0); opacity: 1; }
}
</style>

<template>
  <div class="app">
    <nav class="navbar">
      <div class="nav-brand">
        <span class="logo">📈</span>
        <span class="brand-name">CopyTrade</span>
      </div>
      <div class="nav-links">
        <router-link to="/" class="nav-link">Dashboard</router-link>
        <router-link to="/traders" class="nav-link">Traders</router-link>
        <router-link to="/my-trades" class="nav-link">My Trades</router-link>
      </div>
      <div class="user-info">
        <span class="balance">💰 ${{ currentBalance.toLocaleString() }}</span>
        <span class="username">{{ currentUser }}</span>
      </div>
    </nav>
    <main class="main-content">
      <router-view :key="$route.fullPath"/>
    </main>
  </div>
</template>

<script setup>
import { ref } from 'vue'

const currentUser = ref('NewInvestor')
const currentBalance = ref(10000)
</script>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
  background: #0f1419;
  color: #e7e9ea;
  min-height: 100vh;
}

.app {
  min-height: 100vh;
}

.navbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem 2rem;
  background: #16202a;
  border-bottom: 1px solid #2f3336;
}

.nav-brand {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.logo {
  font-size: 1.5rem;
}

.brand-name {
  font-size: 1.25rem;
  font-weight: 700;
  background: linear-gradient(135deg, #00d4aa 0%, #00a3ff 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.nav-links {
  display: flex;
  gap: 1rem;
}

.nav-link {
  color: #8899a6;
  text-decoration: none;
  padding: 0.5rem 1rem;
  border-radius: 8px;
  transition: all 0.2s;
}

.nav-link:hover,
.nav-link.router-link-active {
  color: #e7e9ea;
  background: #1d2d3a;
}

.user-info {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.balance {
  background: linear-gradient(135deg, #00d4aa22 0%, #00a3ff22 100%);
  padding: 0.5rem 1rem;
  border-radius: 8px;
  color: #00d4aa;
  font-weight: 600;
}

.username {
  color: #8899a6;
}

.main-content {
  padding: 2rem;
  max-width: 1400px;
  margin: 0 auto;
}
</style>

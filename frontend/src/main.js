import { createApp, provide, h } from 'vue'
import { createRouter, createWebHistory } from 'vue-router'
import { ApolloClient, InMemoryCache, createHttpLink } from '@apollo/client/core'
import { DefaultApolloClient } from '@vue/apollo-composable'
import App from './App.vue'

// Components
import Dashboard from './views/Dashboard.vue'
import Traders from './views/Traders.vue'
import MyTrades from './views/MyTrades.vue'

// Apollo Client Setup
const httpLink = createHttpLink({
  uri: '/graphql'
})

const apolloClient = new ApolloClient({
  link: httpLink,
  cache: new InMemoryCache(),
  defaultOptions: {
    query: {
      fetchPolicy: 'no-cache'
    },
    watchQuery: {
      fetchPolicy: 'no-cache'
    }
  }
})

// Router Setup
const routes = [
  { path: '/', name: 'Dashboard', component: Dashboard },
  { path: '/traders', name: 'Traders', component: Traders },
  { path: '/my-trades', name: 'MyTrades', component: MyTrades }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

// Create App
const app = createApp({
  setup() {
    provide(DefaultApolloClient, apolloClient)
  },
  render: () => h(App)
})

app.use(router)
app.mount('#app')

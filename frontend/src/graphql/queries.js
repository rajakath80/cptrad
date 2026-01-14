import gql from 'graphql-tag'

// Queries
export const GET_TRADERS = gql`
  query GetTraders {
    traders {
      id
      username
      balance
      totalPnl
      winRate
      followersCount
      isTrader
      createdAt
    }
  }
`

export const GET_USERS = gql`
  query GetUsers {
    users {
      id
      username
      balance
      totalPnl
      winRate
      followersCount
      isTrader
    }
  }
`

export const GET_TRADES = gql`
  query GetTrades($traderId: ID) {
    trades(traderId: $traderId) {
      id
      traderId
      symbol
      direction
      entryPrice
      exitPrice
      quantity
      pnl
      status
      createdAt
      closedAt
    }
  }
`

export const GET_OPEN_TRADES = gql`
  query GetOpenTrades {
    openTrades {
      id
      traderId
      symbol
      direction
      entryPrice
      quantity
      status
      createdAt
    }
  }
`

export const GET_MY_COPY_RELATIONS = gql`
  query GetMyCopyRelations($followerId: ID!) {
    myCopyRelations(followerId: $followerId) {
      id
      followerId
      traderId
      copyRatio
      active
      createdAt
    }
  }
`

export const GET_MY_COPIED_TRADES = gql`
  query GetMyCopiedTrades($followerId: ID!) {
    myCopiedTrades(followerId: $followerId) {
      id
      originalTradeId
      followerId
      quantity
      pnl
      status
    }
  }
`

// Mutations
export const CREATE_TRADE = gql`
  mutation CreateTrade($input: CreateTradeInput!) {
    createTrade(input: $input) {
      id
      traderId
      symbol
      direction
      entryPrice
      quantity
      status
      createdAt
    }
  }
`

export const CLOSE_TRADE = gql`
  mutation CloseTrade($tradeId: ID!, $exitPrice: Float!) {
    closeTrade(tradeId: $tradeId, exitPrice: $exitPrice) {
      id
      exitPrice
      pnl
      status
      closedAt
    }
  }
`

export const COPY_TRADER = gql`
  mutation CopyTrader($input: CopyTraderInput!) {
    copyTrader(input: $input) {
      id
      followerId
      traderId
      copyRatio
      active
      createdAt
    }
  }
`

export const STOP_COPYING = gql`
  mutation StopCopying($relationId: ID!) {
    stopCopying(relationId: $relationId) {
      id
      active
    }
  }
`

export const REGISTER_USER = gql`
  mutation RegisterUser($username: String!, $isTrader: Boolean!) {
    registerUser(username: $username, isTrader: $isTrader) {
      id
      username
      balance
      isTrader
    }
  }
`

# Context Document: Livestate-POC Demo System

## Behavioral Specification

**Project Goal**: Create a simple, local demonstration system showing how to integrate with the livestate-poc backend for real-time personalization.

**What We're Building**:
1. **Client Library** - JavaScript SDK for tracking user interactions with automatic UUID generation, sequence management, and idempotency
2. **Debugger UI** - Always-visible right panel showing real-time events and state changes
3. **Demo Website** - Single HTML page with action cards representing e-commerce interactions

**Success Criteria**: User can visit local demo, click action cards to simulate e-commerce interactions, and see immediate personalization updates in the debugger panel.

## Architecture Summary

### Backend Integration Points
- **Base URL**: `http://localhost:8788` (local) or deployed Cloudflare Workers
- **Key Endpoints**: `/session/init`, `/session/event`, `/session/state/{cookieId}`
- **Authentication**: Mock system using `cookie-id` header
- **Event Types**: `page_view`, `click`, `purchase`, `search`, `scroll`
- **Response Structure**: Includes propensity scores, user segments, recommendations, next actions

### Client Library Architecture
- **Auto UUID Generation**: Using `crypto.randomUUID()` with localStorage persistence
- **Event Sequencing**: Automatic sequence number management
- **Idempotency**: Timestamp + random string approach
- **Simple API**: `client.trackPageView()`, `client.trackClick()`, etc.
- **Error Handling**: Basic retry logic with exponential backoff

### Debugger UI Design
- **Always-Visible Panel**: Fixed right-side overlay (350px width)
- **Real-Time Updates**: Event stream with timestamps
- **State Visualization**: Propensity scores, segments, recommendations
- **JSON Inspector**: Collapsible API response viewer

### Demo Website Design
- **Single HTML File**: Self-contained demo with embedded CSS and JavaScript
- **Action Cards**: Grid of cards representing e-commerce interactions (View Product, Add to Cart, Search, Checkout, etc.)
- **Click-to-Send**: Each card click sends corresponding event to livestate-poc backend
- **Clean Layout**: Simple, professional design that highlights the real-time personalization

## Research Findings

### JavaScript SDK Best Practices
- **Bundle Size**: Target <20KB for core functionality
- **Browser Support**: ES2018+ with polyfills for older browsers
- **Storage Strategy**: localStorage primary, with memory fallback
- **Network Resilience**: Queue events during failures, retry with backoff

### Demo Website Patterns
- **Action Card Design**: Each card represents a specific e-commerce interaction
- **Event Mapping**: Direct mapping from card clicks to livestate-poc events
- **Visual Feedback**: Cards show interaction state and event confirmation
- **Responsive Grid**: Clean card layout that works on different screen sizes

## Development Constraints

**Scope**: Simple illustration of integration patterns and functionality
**Maturity**: MVP/Prototype level - functional demo with basic error handling
**Deployment**: Local development only, no production deployment needed
**Timeline**: Focus on core working functionality over polish

## Key Implementation Decisions

1. **Monorepo Structure**: Keep all three components in single repository for coordination
2. **Vanilla JavaScript**: Client library as plain JS for broad compatibility
3. **Single HTML Demo**: Self-contained HTML file with action cards for demonstration
4. **Development Focus**: Prioritize working demo over production-ready features
5. **Local Deployment**: All components run locally for demonstration purposes
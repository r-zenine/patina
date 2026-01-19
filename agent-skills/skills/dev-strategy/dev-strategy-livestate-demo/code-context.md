# Code Context: Livestate-POC Demo System

## Key Reference Files from livestate-poc

### Event Type Definitions
**File**: `/live-state-poc/src/types/events.ts`
**Purpose**: Defines the event structure and types that our client library must match
**Key Elements**:
- Event types: `'page_view' | 'click' | 'purchase' | 'search' | 'scroll'`
- Discriminated union for type-safe event data
- Required fields: `cookieId`, `eventType`, `eventData`, `sequenceNumber`, `idempotencyKey`

### API Response Structures
**File**: `/live-state-poc/src/types/api-responses.ts`
**Purpose**: Response formats that our debugger UI needs to display
**Key Elements**:
- `InitialSessionResponse`: Session initialization with past data and tokens
- `EventResponse`: Event processing results with derived state
- `DerivedState`: Propensity scores, segments, recommendations, next actions

### Session Data Types
**File**: `/live-state-poc/src/types/session-data.ts`
**Purpose**: Rich user profile and derived state structures for visualization
**Key Elements**:
- `DerivedState`: Core personalization data to display in debugger
- `UserDemographics`, `UserBehaviorHistory`: Context for understanding user segments
- `PastSessionData`: Historical user data structure

### API Handler Implementation
**File**: `/live-state-poc/src/handlers/LiveStateHandler.ts`
**Purpose**: Server-side endpoint implementations showing expected request/response patterns
**Key Elements**:
- Endpoint routing: `/session/init`, `/session/event`, `/session/state/{cookieId}`
- Request validation patterns
- Error response structures

### Existing Test Client
**File**: `/live-state-poc/tests/live-state-client.ts`
**Purpose**: Reference implementation showing how to integrate with the backend
**Key Elements**:
- Session initialization pattern
- Event processing with sequence numbers
- State retrieval workflow
- Error handling approaches

## Integration Requirements

### Authentication Pattern
```javascript
headers: {
  'Content-Type': 'application/json',
  'cookie-id': cookieId
}
```

### Event Request Structure
Must match the `EventRequest` interface:
- `cookieId`: Auto-generated UUID
- `eventType`: One of the 5 supported types
- `eventData`: Type-specific discriminated union
- `sequenceNumber`: Auto-incrementing integer
- `idempotencyKey`: Unique string for duplicate prevention

### Response Handling
All endpoints return JSON with consistent error structures:
- Success responses include derived state with personalization data
- Error responses include timestamp and status codes
- Gap detection responses include missing sequence ranges

## Demo-Specific Requirements

### Action Card Mapping
Each action card in the HTML demo maps to specific event types:
- **Browse Products** → `page_view` with product catalog URL
- **View Product** → `page_view` with specific product URL
- **Add to Cart** → `click` with cart button element ID
- **Search Products** → `search` with query term
- **Scroll Page** → `scroll` with depth percentage
- **Complete Purchase** → `purchase` with product details

### State Visualization Priority
Focus debugger UI on these key derived state elements:
- `propensityScore`: 0-100 conversion likelihood
- `segment`: User categorization for targeting
- `recommendations`: Array of product IDs
- `nextAction`: Suggested personalization action
- `showPopup`: Boolean for popup trigger

### Real-time Update Flow
1. User clicks action card in demo
2. Client library sends event with auto-generated sequence/idempotency
3. Backend processes event and returns derived state
4. Debugger UI immediately updates with new personalization data
5. Visual feedback shows the impact of the user's action

This creates a clear demonstration of how each user interaction drives immediate personalization changes through the livestate-poc backend.
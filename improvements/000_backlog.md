Workstream 1: Context folding
- enrich the fixture in main.rs to include  richer kind of change including  ( in order to test folding capabilities )
        pub const ESSENTIAL: RelevanceScore = 0; // Contains or is the actual change
        pub const IMPORTANT: RelevanceScore = 1; // Direct semantic container of change  
        pub const BACKGROUND: RelevanceScore = 2; // Sibling context (collapsible in UI)
        pub const NOISE: RelevanceScore = 3; // Unrelated context (hideable in UI)
- check that context folding works

Workstream 2: Approvals
- ticker not showing up after approval at check level ( no visual feedback )
- Approval system seems britle

Workstream 3: Simplify key bindings
- remove (<Space>e) and the associated code and capability
- remove <Space>d altogether
- remove <Space>c altogether

Workstream 4: Simplify UI
- remove from `[Enter] expand files  [j/k] navigate  [Space] actions` from decision display widget
- Make colorscheme more readable

Workstream 5 : Mist
- '?' not working
- `📄 reader.rs | 0/0 approved | Overall: 3/30 | Full Context` out of sync with the state

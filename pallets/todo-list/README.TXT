# Todo List Pallet - Substrate Node Template Project

## 1. Clone and Setup Substrate Node Template

First, clone the Substrate Node Template and set up your development environment:

```bash
# Clone the node template
git clone https://github.com/paritytech/polkadot-sdk-parachain-template.git parachain-template
cd substrate-node-template

# Build the node
cargo build --release

# Run the node in development mode
./target/release/node-template --dev
```

Expected output: A running Substrate node with development configuration.

## 2. Create Custom Pallet: pallet-todo-list

### Pallet Structure
Create a new pallet in the `pallets` directory:

```bash
cd pallets
cargo new --lib todo-list
```

### Dependencies
Add the following to `pallets/todo-list/Cargo.toml`:

```toml
[package]
name = "pallet-todo-list"
version = "0.1.0"
edition = "2021"

[dependencies]
frame-support = { version = "4.0.0", default-features = false }
frame-system = { version = "4.0.0", default-features = false }
sp-runtime = { version = "7.0.0", default-features = false }
sp-std = { version = "7.0.0", default-features = false }

[features]
default = ["std"]
std = [
    "frame-support/std",
    "frame-system/std",
    "sp-runtime/std",
    "sp-std/std"
]
```

### Implementation
The pallet implements the following features:

#### Storage
```rust
#[pallet::storage]
pub type Todos<T: Config> = StorageMap<_, Blake2_128Concat, u32, (Vec<u8>, T::AccountId, bool), OptionQuery>;

#[pallet::storage]
pub type TodoCount<T> = StorageValue<_, u32, ValueQuery>;
```

#### Events
```rust
#[pallet::event]
pub enum Event<T: Config> {
    TaskCreated(u32, T::AccountId, Vec<u8>),
    TaskCompleted(u32, T::AccountId),
}
```

#### Dispatchable Functions
1. `create_task(description: Vec<u8>)`
   - Creates a new task with auto-increment ID
   - Sets owner as caller
   - Emits TaskCreated event

2. `complete_task(task_id: u32)`
   - Marks task as complete
   - Verifies caller is the task owner
   - Emits TaskCompleted event

### Runtime Integration

1. Add to `runtime/Cargo.toml`:
```toml
[dependencies]
pallet-todo-list = { path = "../pallets/todo-list" }
```

2. Add to `runtime/src/lib.rs`:
```rust
#[runtime::pallet_index(53)]
pub type TodoList = pallet_todo_list;

impl pallet_todo_list::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type TodoCount = ConstU32<100>;
}
```

3. Add to `construct_runtime!`:
```rust
construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = opaque::Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        // ... other pallets ...
        TodoList: pallet_todo_list,
    }
);
```

## 3. Running and Testing the Pallet

1. Build the node:
```bash
cargo build --release
```

2. Run the node:
```bash
chain-spec-builder create -t development --relay-chain paseo --para-id 1000 --runtime ./target/release/wbuild/parachain-template-runtime/parachain_template_runtime.compact.compressed.wasm named-preset development
```bash
polkadot-omni-node --chain ./chain_spec.json --dev
```

3. Using Polkadot.js Apps:
   - Connect to your local node (ws://127.0.0.1:9944)
   - Navigate to Developer > Extrinsics
   - Create a task using `todoList.createTask`
   - Complete the task using `todoList.completeTask`
   - Verify task completion in Developer > Chain State

### Expected Results
- Successful task creation with auto-increment ID
- Task completion by owner
- Storage verification showing completed status
- Event emission for both operations

## Verification Steps
1. Check successful build with no errors
2. Verify pallet is included in runtime
3. Test core operations through Polkadot.js Apps
4. Confirm storage updates and event emissions

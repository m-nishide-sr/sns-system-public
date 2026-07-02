# AI Agent Instructions

You MUST read and follow these files before working in this directory:
- [core.md](../docs/core.md) (specs)
- [README.md](../README.md) (Root project guidelines)

Append points requiring consideration, inconsistencies, unresolved issues, and other TODOs to [TODO.md](./TODO.md).

# AI Agent Guidelines for `/core` (Core Business Logic)

When generating, modifying, or reviewing code within this directory (`/core`), you must adhere to the following design principles and layer definitions as **absolute constraints**.

---

## 1. Common Design Principles (Architectural Guardrails)

- **Dependency Direction (Strictly Enforced)**:
Dependencies must always flow inward: `domain` <- `usecase` <- `infrastructure`. Outer layers may depend on inner layers, but inner layers are strictly prohibited from importing from outer layers.
- **Maximizing Rust Type Safety**:
Do not handle raw primitive types (e.g., `String`, `i32`) directly in the domain layer. Instead, define them as "immutable types" (using the Newtype pattern) that perform validation within their constructors.
- **Eliminating Framework and External Library Dependencies**:
The `domain` and `usecase` layers must have absolutely no knowledge of specific web frameworks (e.g., Lambda) or ORMs (e.g., SeaORM).

---

## 2. Layer Definitions and Strict Rules

### 2.1. Domain Layer (`/core/domain/`)
The innermost layer of the system, encapsulating the purest business rules (domain knowledge).

- **Import Restrictions**:
- Must never import code or crates from `usecase`, `infrastructure`, or `/db/sea_orm_entities`. 
- Dependencies on external crates (libraries) are prohibited in principle (with the exception of standard, general-purpose tools like `serde` or time-handling libraries).
- **Content to Implement**:
- **Value Objects (Newtypes)**: Immutable structs that encapsulate validation logic. 
- **Entities**: Domain objects identified by an ID that undergo state changes. - **Domain Rules (Behavior)**: Logic for data validation and state transitions is encapsulated as methods (`pub fn`) on structs.
- **Repository Trait (Interface)**: Uses `async-trait` to abstract data persistence and retrieval.

### 2.2. Usecase Layer (`/core/usecase/`)
The layer that controls application execution procedures (scenarios).

- **Import Restrictions**:
- Only `domain` and `common` may be imported. 
- `infrastructure` and `/db/sea_orm_entities` must not be imported.
- **Implementation Details**:
- **Use Cases (Interactors)**: Define procedures that accept input (DTOs), retrieve or manipulate entities via Domain layer repository traits, and return results. 
- **Input/Output Data Structures (DTOs)**: Pure data structures for exchanging data with outer layers (e.g., Lambda functions).
- **Note**: Business rules themselves must not be implemented in this layer; instead, call methods defined in the Domain layer.

### 2.3. Infrastructure Layer (`/core/infrastructure/`)
The layer that implements concrete communication with external details (databases, external APIs, etc.).

- **Import Restrictions**:
- `domain`, `usecase`, and the external `/db/sea_orm_entities` may be imported.
- **Implementation Details**:
- **Concrete Repository Implementations**: Concrete implementations of the repository traits defined in the Domain layer, using tools like `SeaORM`. 
- **Data Model Conversion (Mapping)**: Logic to convert between database models (from `sea_orm_entities`) and Domain layer objects (Entities/Value Objects)—e.g., by implementing the `From` trait.

### 2.4. Common Layer (`/core/common/`)
General-purpose utilities used across multiple layers. - **Important Note**: Do not write business logic (domain knowledge) here. Place only "technical helpers"—such as loggers, common error types, or date/time manipulation utilities—in this location.

---

## 3. Documentation and Comments (Cargo Doc Standard)

Every struct, trait, enum, and public function must be documented using Rust's standard documentation comments (`///` or `//!`) so they are properly rendered via `cargo doc`.

- **Document the "WHY", not the "WHAT"**: 
  Do not explain *what* the code does (the code itself should be self-explanatory). Detailed explanations must focus on *why* a particular design, business constraint, or logic was chosen.
- **Provide Context**: 
  Include edge cases, business background, or relationships with external systems to make the generated `cargo doc` a living knowledge base for human developers.

---
name: rust-errors
description: Type-safe error handling patterns for passing structured Rust errors to TypeScript in Tauri applications. Use for Tauri error handling and cross-language error propagation.
---

# Rust to TypeScript Error Handling

This document outlines a discriminated union pattern for safely transmitting errors from Rust to TypeScript in Tauri applications.

## Core Approach

The strategy uses internally-tagged enums with `#[serde(tag = "name")]` to create a consistent error structure. As the guide explains, this produces errors where "all errors have the same shape with `name` and `message`" fields, making them straightforward to handle in TypeScript.

## Implementation Overview

**Rust side:** Define error enums with the `thiserror` crate, deriving both `Serialize` and `Deserialize`. Each variant includes a message field.

**TypeScript side:** Use arktype for runtime validation, then employ switch statements to handle specific error cases with full type safety.

## Key Benefits

- **Consistency:** Every error follows an identical structure
- **Type Safety:** Runtime validation catches unexpected error shapes before processing
- **Exhaustiveness:** Switch statements ensure all cases are handled

## Critical Warnings

Avoid external tagging (the default Rust behavior), which produces nested structures incompatible with this pattern. Also skip adjacent tagging with content attributes, as these create unnecessary nesting that complicates TypeScript consumption.

## Rust Error Pattern

```rust
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Serialize, Deserialize)]
#[serde(tag = "name")]
pub enum AppError {
    #[error("Network error: {message}")]
    NetworkError { message: String },

    #[error("Validation error: {message}")]
    ValidationError { message: String },

    #[error("Not found: {message}")]
    NotFound { message: String },
}
```

## TypeScript Handling with Arktype

```typescript
import { type } from 'arktype';

const AppErrorSchema = type({
    name: "'NetworkError' | 'ValidationError' | 'NotFound'",
    message: "string"
});

function handleError(error: unknown) {
    const result = AppErrorSchema(error);
    if (result.problems) {
        console.error('Unknown error shape:', error);
        return;
    }

    switch (result.name) {
        case 'NetworkError':
            // Handle network errors
            break;
        case 'ValidationError':
            // Handle validation errors
            break;
        case 'NotFound':
            // Handle not found errors
            break;
    }
}
```

## Best Practices

1. **Always use `#[serde(tag = "name")]`** - This creates flat, predictable JSON
2. **Include a `message` field on every variant** - Provides human-readable context
3. **Use `thiserror` for the Error derive** - Gets you proper Display implementation
4. **Validate at the boundary** - Use arktype or zod before processing
5. **Exhaustive switches** - TypeScript will warn if you miss a case

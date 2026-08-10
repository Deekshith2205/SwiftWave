# Tests

Integration and end-to-end tests for SwiftWave.

## Structure

```
tests/
  (future: workspace-level integration tests)
```

## Running Tests

### Rust unit + integration tests
```bash
cargo test --all
```

### Flutter widget + unit tests
```bash
cd apps/flutter_app
flutter test
```

### End-to-end tests (Phase 5)
Full E2E tests require two physical devices or emulators on the same Wi-Fi segment.
Instructions will be added in Phase 5.

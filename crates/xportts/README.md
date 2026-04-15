# xportts

TypeScript/WASM bindings for [xportrs](https://github.com/rubentalstra/xportrs) — a CDISC-compliant XPT file generation and parsing library.

## Usage

```typescript
import { to_xpt, Dataset } from "xportts";

const dataset = Dataset.new("AE", []);
const bytes = to_xpt(dataset);
```

## License

MIT

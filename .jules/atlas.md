**The Leaky Abstraction: Stringly Typed Accounts**
**Tangle:** Accounts were represented as naked `String` types across `RsuDistributorConfig` and other configuration models, leading to weakly enforced domain boundaries and potential bugs (e.g. passing a category name instead of an account name or parameter mix-ups).
**Blueprint:** Introduced the `AccountId(String)` New Type in the `domain::account` module to strongly type account references, specifically starting with configurations like `RsuDistributorConfig` to prevent stringly-typed configuration smells.

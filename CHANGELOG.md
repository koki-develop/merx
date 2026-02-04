# Changelog

## [0.1.4](https://github.com/koki-develop/merx/compare/v0.1.3...v0.1.4) (2026-02-04)


### Bug Fixes

* Accept &str in Environment::set() to avoid String clone on reassignment ([0b5ff6a](https://github.com/koki-develop/merx/commit/0b5ff6abf8e718e1207f96e74d0b859ea92bacb4))
* Replace HashMap SipHash with FxHashMap for faster variable lookup ([826af9e](https://github.com/koki-develop/merx/commit/826af9ee29867a7d738307364004d2284a6115c6))
* Return Cow&lt;Value&gt; from eval_expr to eliminate variable lookup clones ([e5b4ba4](https://github.com/koki-develop/merx/commit/e5b4ba47c4476be62d36c2aed399b8e2343ed163))

## [0.1.3](https://github.com/koki-develop/merx/compare/v0.1.2...v0.1.3) (2026-02-04)


### Bug Fixes

* Remove unnecessary Node clone in interpreter loop ([ab80829](https://github.com/koki-develop/merx/commit/ab80829bd30c35cf0d6e5e2d021f14d143974a60))
* Replace HashMap with Vec+usize index for node/edge lookup ([2a5f56f](https://github.com/koki-develop/merx/commit/2a5f56f51ad2689567c50d0f7914bb933845a96f))

## [0.1.2](https://github.com/koki-develop/merx/compare/v0.1.1...v0.1.2) (2026-02-04)


### Features

* Release v0.1.2 ([04b31dc](https://github.com/koki-develop/merx/commit/04b31dcd4b727b4485c26c7b28b942a63b5b402c))

## [0.1.1](https://github.com/koki-develop/merx/compare/v0.1.0...v0.1.1) (2026-02-03)


### Features

* Release v0.1.1 ([d51a517](https://github.com/koki-develop/merx/commit/d51a5176c634288478151efab60da97aff7d8f58))

## [0.1.0](https://github.com/koki-develop/merx/compare/v0.0.4...v0.1.0) (2026-02-03)


### Features

* Add escape sequence support for string literals ([9eb0f7a](https://github.com/koki-develop/merx/commit/9eb0f7a2e414229aed2a657d5e51e0de577b07ce))
* Add escape sequence support for string literals ([6fd27d6](https://github.com/koki-develop/merx/commit/6fd27d6384be57599e5ee5fd20e5788e20ad8d2a))
* Add optional double-quote support for node labels ([7d601ca](https://github.com/koki-develop/merx/commit/7d601ca33fa7c50b64bfd694fcd7f75cef480303))
* Allow whitespace in edge labels ([4ff6835](https://github.com/koki-develop/merx/commit/4ff68356d7c7298c23f75d25227dbb5facbba202))
* Detect duplicate node definitions at parse time ([034f2b8](https://github.com/koki-develop/merx/commit/034f2b868ae3e65122a058f44e6f1d961dc411aa))
* Detect undefined node references at parse time ([e8465ed](https://github.com/koki-develop/merx/commit/e8465ed6e8045a37396a6f69015b1d4567fa5a57))
* Propagate I/O errors from OutputWriter instead of silently ignoring them ([e791537](https://github.com/koki-develop/merx/commit/e7915372054287b5fe0609809c7ecb16227160ad))
* Support arbitrary-length arrows in edge definitions ([e7aa496](https://github.com/koki-develop/merx/commit/e7aa496b260f2d429cefd05320c9f66b73776326))
* Support exit code specification via edge labels to End node ([55eb116](https://github.com/koki-develop/merx/commit/55eb116b402af6a46747394913d529c47c45dc00))
* Support inline label syntax (--text--&gt;) for edge definitions ([4c676b4](https://github.com/koki-develop/merx/commit/4c676b48597a906eb0e229b729d9fa969a7ce619))
* Support string concatenation with the `+` operator ([6050175](https://github.com/koki-develop/merx/commit/6050175ed2d0fbf42a3d4c3ad555559ef41950a6))
* Validate Start/End node existence at parse time ([a180bc1](https://github.com/koki-develop/merx/commit/a180bc1925313d2303c8564a1f81cc64039391f3))

## [0.0.4](https://github.com/koki-develop/merx/compare/v0.0.3...v0.0.4) (2026-02-03)


### Features

* Add optional label support for Start/End nodes ([5473f89](https://github.com/koki-develop/merx/commit/5473f896b4b8956f2bedd57b205aa65d7dd77db7))
* Release v0.0.4 ([5aefc78](https://github.com/koki-develop/merx/commit/5aefc78f00a9d4786dacf764add8fe970da07059))

## [0.0.3](https://github.com/koki-develop/merx/compare/v0.0.2...v0.0.3) (2026-02-03)


### Features

* Release v0.0.3 ([17989f9](https://github.com/koki-develop/merx/commit/17989f9f85152137bd79b1d0ffb3aa987bb0c730))

## [0.0.2](https://github.com/koki-develop/merx/compare/v0.0.1...v0.0.2) (2026-02-03)


### Features

* Release v0.0.2 ([3cf9c5e](https://github.com/koki-develop/merx/commit/3cf9c5e698c68369d83db8a17a7419b1592e4f9b))

## 0.0.1 (2026-02-02)


### Features

* Add CLI interface with clap ([4606573](https://github.com/koki-develop/merx/commit/46065738646b7064f7af66009b2e8ae76187332b))
* Add Mermaid comment syntax support (`%%`) ([954ac79](https://github.com/koki-develop/merx/commit/954ac79d4ec220e752d7462c8ffb34d1cdaae0ee))
* Add pest parser for Mermaid flowchart syntax ([430068c](https://github.com/koki-develop/merx/commit/430068cb5110417df9a4513a3304ba2e23cd6b7f))
* Add print statement (no newline version of println) ([e7b5535](https://github.com/koki-develop/merx/commit/e7b55351f04ae9b5544dc50e43f50b2d20c6bcf5))
* Add runtime execution engine for Mermaid flowcharts ([978e823](https://github.com/koki-develop/merx/commit/978e823a1aaa821b752bc72ed49f8ce279798dd5))
* Add validation for multiple outgoing edges from non-condition nodes ([281dd68](https://github.com/koki-develop/merx/commit/281dd681ae1a4633781127af486d9dfb95fff940))
* Add validation to prevent outgoing edges from End node ([abc38a7](https://github.com/koki-develop/merx/commit/abc38a7eaa31b78ce62e17274ad6060ff0257e3f))
* Release v0.0.1 ([38d6b6f](https://github.com/koki-develop/merx/commit/38d6b6ff85aaa8f6385e169aaf70c85636b885fc))
* Rename print statement keyword to println ([725e401](https://github.com/koki-develop/merx/commit/725e401f5fc604c3f90e2e535c83fd16d89d7ee8))

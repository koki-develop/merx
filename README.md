<h1 align="center">Merx</h1>
<p align="center">
<i>Run your flowcharts.</i>
</p>

<p align='center'>
<a href="https://github.com/koki-develop/merx/releases/latest"><img alt="GitHub release (latest by date)" src="https://img.shields.io/github/v/release/koki-develop/merx?style=flat"></a>
<a href="./LICENSE"><img src="https://img.shields.io/github/license/koki-develop/merx?style=flat" /></a>
<a href="https://github.com/koki-develop/merx/actions/workflows/ci.yml"><img alt="GitHub Workflow Status" src="https://img.shields.io/github/actions/workflow/status/koki-develop/merx/ci.yml?branch=main&logo=github&style=flat" /></a>
</p>

<p align="center">
Merx is an interpreter that executes programs written in Mermaid flowchart syntax.
</p>

```mermaid
%% hello.mmd
flowchart TD
    Start --> A[print 'Hello, merx!']
    A --> End
```

```console
$ merx run hello.mmd
Hello, merx!
```

## Installation

### Homebrew

```sh
brew install koki-develop/tap/merx
```

### GitHub Releases

Download the binary from [GitHub Releases](https://github.com/koki-develop/merx/releases).

## Examples

See the [examples](./examples) directory.

## License

[MIT](./LICENSE)

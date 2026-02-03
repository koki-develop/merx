<h1 align="center">Merx</h1>
<p align="center">
<i>Run your flowcharts.</i>
</p>

<p align='center'>
<a href="https://github.com/koki-develop/merx/releases/latest"><img alt="GitHub release (latest by date)" src="https://img.shields.io/github/v/release/koki-develop/merx?style=flat"></a>
<a href="./LICENSE"><img src="https://img.shields.io/github/license/koki-develop/merx?style=flat" /></a>
<a href="https://github.com/koki-develop/merx/actions/workflows/ci.yml"><img alt="GitHub Workflow Status" src="https://img.shields.io/github/actions/workflow/status/koki-develop/merx/ci.yml?branch=main&logo=github&style=flat" /></a>
<a href="https://codecov.io/gh/koki-develop/merx" ><img src="https://codecov.io/gh/koki-develop/merx/graph/badge.svg?token=H2WG9FO7A4"/></a>
</p>

<p align="center">
Merx is an interpreter that executes programs written in Mermaid flowchart syntax.
</p>

```mermaid
%% hello.mmd
flowchart TD
    Start([Start]) --> A[println 'Hello, merx!']
    A --> End([End])
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

## Documentation

For more details, see the [documentation](https://koki-develop.github.io/merx/).

## License

[MIT](./LICENSE)

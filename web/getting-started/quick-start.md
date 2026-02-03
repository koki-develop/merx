# Quick Start

## Hello, merx!

Create a file named `hello.mmd` with the following content:

```mmd
flowchart TD
    Start([Start]) --> A[println 'Hello, merx!']
    A --> End([End])
```

Run it:

```console
$ merx run hello.mmd
Hello, merx!
```

## How it works

Merx executes Mermaid flowcharts as programs. The flowchart is traversed from the `Start` node to the `End` node, executing statements in each node along the way.

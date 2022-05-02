<div style="text-align: center">
	<h1>faas-watchdog-wasmer-gpu</h1>
	<p>
    <a href="https://github.com/yanghaku/faas-watchdog-wasmer-gpu/blob/main/LICENSE">
	    <img src="https://img.shields.io/badge/license-Apache-brightgreen" alt="License">
    </a>
	<img src="https://img.shields.io/badge/test-passing-brightgreen" alt="Test Status">
	<img src="https://img.shields.io/badge/status-developing-brightgreen" alt="Status">
	</p>
</div>
<hr/>

The of-watchdog implements an HTTP server listening on port 8080, and acts as a reverse proxy for running functions and
microservices. It can be used independently, or as the entrypoint for a container with OpenFaaS.

This version can run **webassembly** with [```wasmer-gpu```](https://github.com/yanghaku/wasmer-gpu) implemented
in ```rust```.

The behavior is same as [```of-watchdog```](https://github.com/openfaas/of-watchdog), and add the mode ```wasm``` to
support new feature.

## Wasm Mode (mode=wasm) (default)

Running WebAssembly instance in thread pool, a function is served as a thread in process.

* Compute resource isolation
    * CPU: now only use one thead in thread pool to run functions.
    * Memory: strong memory isolation. ***todo:*** 64bit memory support
    * GPU: now it can use cuda.  ***todo:*** resource limitation

* FileSystem
    * use **```wasm_root```** as file system root for webassembly liking ```chroot```.
    * when multi webassembly instances access the same file in ```wasm_root```,
      we use the **```copy on write```** strategy liking ```fork```

* Network
    * ***pending***

## Configuration

For the full configuration you can see in [```watchdog```](https://github.com/openfaas/of-watchdog#configuration)

The extra environment variable for ```wasm``` mode:

| key                       | description                                                    | default      |
|---------------------------|----------------------------------------------------------------|--------------|
| **```wasm_root```**       | The file system root for webassembly instance                  | ```/```      |
| **```use_cuda```**        | If enable cuda support                                         | ```false```  |
| **```min_scale```**       | min replicas for function instances, also is the init replicas | ```1```      |
| **```max_scale```**       | max replicas for function instances                            | ```4096```   |
| ```wasm_c_target```       | (```compiler``` feature only) compile target                   | host target  |
| ```wasm_c_cpu_features``` | (```compiler``` feature only) compile target cpu features      | host default |

## example

You can download some example wasm module file
in [```wasm-cuda-simple-examples```](https://github.com/yanghaku/wasm-cuda-simple-examples)

Then run:

```shell
fprocess=/wasm_root/bin/device use_cuda=true ./watchdog
```

## Status:

| mode          | status                                                 |
|---------------|--------------------------------------------------------|
| "streaming"   | ![pending](https://img.shields.io/badge/-pending-blue) |
| "serializing" | ![pending](https://img.shields.io/badge/-pending-blue) |
| "http"        | ![pending](https://img.shields.io/badge/-pending-blue) |
| "static"      | ![pending](https://img.shields.io/badge/-pending-blue) |
| "**wasm**"    | ![OK](https://img.shields.io/badge/-OK-brightgreen)    |

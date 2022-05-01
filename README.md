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

Openfass watchdog which can run webassembly with wasmer-gpu implemented in ```rust```.

The behavior is same as [```of-watchdog```](https://github.com/openfaas/of-watchdog), and add the mode ```wasm``` to
support new feature.

## Status:

| mode          | status                                                 |
|---------------|--------------------------------------------------------|
| "streaming"   | ![pending](https://img.shields.io/badge/-pending-blue) |
| "serializing" | ![pending](https://img.shields.io/badge/-pending-blue) |
| "http"        | ![pending](https://img.shields.io/badge/-pending-blue) |
| "static"      | ![pending](https://img.shields.io/badge/-pending-blue) |
| "**wasm**"    | ![OK](https://img.shields.io/badge/-OK-brightgreen)    |

## Isolation

* Compute resource isolation
    * CPU: now only use one thead in thread pool to run functions. ***todo:*** multi thread and other controller
      using ```cgroup```
    * Memory: strong memory isolation. ***todo:*** 64bit memory support
    * GPU: now it can use cuda.  ***todo:*** resource limitation

* FileSystem
    * use **```wasm_root```** as file system root for webassembly liking ```chroot```.
    * when multi webassembly instances access the same file in ```wasm_root```,
      we use the **```copy on write```** strategy liking ```fork```

* Network
    * ***pending***

## Usage

The extra environment variable:

* **```wasm_root```**: the file system root for webassembly instance, default is ```/```
* **```use_cuda```**: if enable cuda support, default is ```false```
* **```max_scale```**: max running function number, default is the number of cpu cores
* ```wasm_c_target```: compile target, default is host target
* ```wasm_c_cpu_features```: compile target cpu features, default is host default

example

```shell
fprocess=/wasm_root/bin/device use_cuda=true ./watchdog
```

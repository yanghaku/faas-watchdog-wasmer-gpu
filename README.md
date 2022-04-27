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

Status:

| mode          | status                                                 |
|---------------|--------------------------------------------------------|
| "streaming"   | ![pending](https://img.shields.io/badge/-pending-blue) |
| "serializing" | ![pending](https://img.shields.io/badge/-pending-blue) |
| "http"        | ![pending](https://img.shields.io/badge/-pending-blue) |
| "static"      | ![pending](https://img.shields.io/badge/-pending-blue) |
| "**wasm**"    | ![OK](https://img.shields.io/badge/-OK-brightgreen)    |

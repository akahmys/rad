package main

import (
	"fmt"
	// The import path below assumes bindings are generated in the "gen" directory
	// using: wit-bindgen-go generate ./wit
	"github.com/akahmys/rad/templates/go/gen/radcomp/extension/types"
)

// Define the Extension handler implementing the guest interface exports.
type Extension struct{}

func init() {
	// Register the handler to the generated Wasm component bindings
	types.RegisterRadExtension(Extension{})
}

// OnEvent is called by RAD Core when dispatching events to the Wasm component.
func (e Extension) OnEvent(event types.RasCoreEvent) types.Result[struct{}, string] {
	switch ev := event.(type) {
	case types.RasCoreEventHumanInputReceived:
		fmt.Printf("Go Template Extension received prompt: %s\n", ev.Value)
		// Call host capabilities if needed:
		// res := types.HostRpc(types.RasRpcCommandWriteStdout{Value: "Hello from Go!"})
	case types.RasCoreEventTaskCompleted:
		fmt.Println("Go Template Extension notified of task completion.")
	}
	
	// Return empty success result
	return types.Ok[struct{}, string](struct{}{})
}

// VerifyRpc is a security hook called by RAD Core to inspect and approve/deny actions.
func (e Extension) VerifyRpc(command types.RasRpcCommand) bool {
	switch cmd := command.(type) {
	case types.RasRpcCommandSpawnBashProcess:
		fmt.Printf("Go Verifying bash execution: %s\n", cmd.Value)
		return true
	default:
		return true
	}
}

// A main function is required for TinyGo compilation into a WASI binary.
func main() {}

// Learn more about F# at http://fsharp.org
open System
open TraceServer.Generator

[<EntryPoint>]
let main argv =
    match argv with
    | [| trace_file |] -> 
        generateBasicBlock trace_file
        1
    | _ -> 
        printfn "trace file is not given"
        0

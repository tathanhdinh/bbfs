// Learn more about F# at http://fsharp.org
open System
open TraceServer.Generator

[<EntryPoint>]
let main argv =
    match argv with
    | [| trace_file |] -> 
        let (translatedBasicBlock, allBasicBlock) = generateBasicBlock trace_file
        printfn "\n\ntranslated basic blocks: %i (total: %i)" translatedBasicBlock allBasicBlock
        1
    | _ -> 
        printfn "trace file is not given"
        0

open Argu
open ShellProgressBar
open Cache.Generator

type CLIArguments =
    | [<Mandatory>] Trace of trace : string
    | [<Mandatory>] Metadata of metadata : string
    interface IArgParserTemplate with
        member arg.Usage =
            match arg with
            | Trace _ -> "specify a trace file"
            | Metadata _ -> "specify a metadata file"

[<EntryPoint>]
let main argv =
    let argParser = new ArgumentParser<CLIArguments>()
    try
        let results = argParser.ParseCommandLine(inputs = argv, raiseOnUsage = true)
        let traceFile = results.GetResult Trace
        let metadataFile = results.GetResult Metadata
        let basicBlockGenerator = BasicBlockGenerator(traceFile, metadataFile)
        let totalBasicBlockCount = basicBlockGenerator.TotalBasicBlockCount
        let mutable progressBarOption = new ProgressBarOptions()
        progressBarOption.CollapseWhenFinished <- true
        let progressBar =
            new ProgressBar(int totalBasicBlockCount, "trace cached", progressBarOption)
        let mutable readCount = 0
        try
            while true do
                basicBlockGenerator.generate()
                readCount <- readCount + 1
                progressBar.Tick(readCount)
        with _ -> ()
        printfn "\n\nraw basic blocks: %i (total: %i)" basicBlockGenerator.RawBasicBlockCount
            basicBlockGenerator.GeneratedBasicBlockCount
        1
    with e ->
        printfn "%s" e.Message
        0

namespace TraceServer

open System.IO
open System.Runtime.Serialization.Formatters.Binary
open StackExchange.Redis
open K4os.Compression.LZ4.Streams
open ShellProgressBar

type BasicBlock =
    { ProgramCounter : uint64
      ExecutionMode : uint8
      Data : byte array
      LoopCount : uint64 }

module Generator =
    [<Literal>]
    let BasicBlockDataList = "raw_basic_block_list"
    
    [<Literal>]
    let BasicBlockList = "basic_block_list"
    
    let generateBasicBlock (traceFile : string) =
        // use traceFileReader = new BinaryReader(File.OpenRead(traceFile))
        let decoderStream = LZ4Stream.Decode(File.OpenRead(traceFile))
        use traceFileReader = new BinaryReader(decoderStream)
        let redisConnection = ConnectionMultiplexer.Connect("localhost")
        let redisDB = redisConnection.GetDatabase()
        let binaryFormatter = new BinaryFormatter()

        let mutable progressBarOption = new ProgressBarOptions()
        // progressBarOption.ProgressCharacter <- '\u2593'
        progressBarOption.CollapseWhenFinished <- true
        let progressBar = new ProgressBar(850590, "trace cached", progressBarOption)
        
        let serializeBasicBlock basicBlock =
            use stream = new MemoryStream()
            binaryFormatter.Serialize(stream, basicBlock)
            stream.ToArray()
        
        // ref: https://stackoverflow.com/questions/27274702/how-do-i-call-redis-stringset-from-f
        let inline (~~) (x : ^a) : ^b = ((^a or ^b) : (static member op_Implicit : ^a -> ^b) x)
        
        let readBasicBlock() =
            let isBasicBlockTranslated = traceFileReader.ReadByte()
            match isBasicBlockTranslated with
            | 0uy -> 
                (* basic block is not translated yet *)
                let basicBlockRawSize = traceFileReader.ReadUInt16()
                let basicBlockData = traceFileReader.ReadBytes(int basicBlockRawSize)
                let basicBlockProgramCounter = traceFileReader.ReadUInt64()
                let basicBlockExecutionMode = traceFileReader.ReadByte()
                let basicBlockLoopCount = traceFileReader.ReadUInt64()
                
                let basicBlock =
                    { ProgramCounter = basicBlockProgramCounter
                      ExecutionMode = basicBlockExecutionMode
                      Data = basicBlockData
                      LoopCount = basicBlockLoopCount }
                
                // ref: https://github.com/StackExchange/StackExchange.Redis/issues/831
                let basicBlockDataIndex =
                    redisDB.ListRightPush(~~BasicBlockDataList, ~~basicBlockData) |> ignore
                // ref: https://gist.github.com/theburningmonk/2071722
                let serializedBasicBlock = serializeBasicBlock basicBlock
                redisDB.ListRightPush(~~BasicBlockList, ~~serializedBasicBlock) |> ignore
            | 1uy -> 
                (* basic block has been translated *)
                let basicBlockIndex = traceFileReader.ReadUInt64()
                let basicBlockData =
                    ~~redisDB.ListGetByIndex(~~BasicBlockDataList, int64 basicBlockIndex)
                let basicBlockProgramCounter = traceFileReader.ReadUInt64()
                let basicBlockExecutionMode = traceFileReader.ReadByte()
                let basicBlockLoopCount = traceFileReader.ReadUInt64()
                
                let basicBlock =
                    { ProgramCounter = basicBlockProgramCounter
                      ExecutionMode = basicBlockExecutionMode
                      Data = basicBlockData
                      LoopCount = basicBlockLoopCount }
                
                let serializedBasicBlock = serializeBasicBlock basicBlock
                redisDB.ListRightPush(~~BasicBlockList, ~~serializedBasicBlock) |> ignore
            | _ -> failwith "unreachable"
        let mutable readCount = 0
        try 
            while true do
                readBasicBlock()
                readCount <- readCount + 1
                progressBar.Tick(readCount)
        with _ -> ()
        let dataListLength = redisDB.ListLength(~~BasicBlockDataList)
        let basicBlockListLength = redisDB.ListLength(~~BasicBlockList)
        (dataListLength, basicBlockListLength)

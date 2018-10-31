namespace Cache

open System.IO
open System.Runtime.Serialization.Formatters.Binary
open StackExchange.Redis
open K4os.Compression.LZ4.Streams
open System

type BasicBlock =
    { ProgramCounter : uint64
      ExecutionMode : uint8
      Privilege : uint8
      Data : byte array
      LoopCount : uint64 }

module Generator =
    let [<Literal>] BasicBlockDataList = "raw_basic_block_list"

    let [<Literal>] BasicBlockList = "basic_block_list"

    let [<Literal>] AddressIndependentBasicBlockList = "address_independent_basic_block_list"

    type BasicBlockGenerator (traceFile: string, metadataFile: string, ?cacheServer: string) =
        let addressIndependentBasicBlockHashes = Set.empty
        let traceDataStream =
            let decoderStream = LZ4Stream.Decode <| File.OpenRead traceFile
            new BinaryReader(decoderStream)

        let redisDatabase =
            let server = defaultArg cacheServer "localhost"
            let connection = ConnectionMultiplexer.Connect server
            connection.GetDatabase ()

        let serializingBinaryFormatter = BinaryFormatter ()

        member gen.TotalBasicBlockCount =
            use metaDataFileReader = new BinaryReader(File.OpenRead metadataFile)
            metaDataFileReader.ReadUInt64 ()

        member gen.generate () =
            // ref: https://stackoverflow.com/questions/27274702/how-do-i-call-redis-stringset-from-f
            let inline (~~) (x : ^a) : ^b = ((^a or ^b) : (static member op_Implicit : ^a -> ^b) x)

            let serializeBasicBlock basicBlock =
                // let basicBlockSize = sizeof<uint64> + // ProgramCounter
                //                      sizeof<uint8> +  // ExecutionMode
                //                      sizeof<uint8> +  // Privilege
                //                      sizeof<uint64> + // LoopCount
                //                      Array.length basicBlock.Data

                Array.concat [| BitConverter.GetBytes(basicBlock.ProgramCounter);
                                [| basicBlock.ExecutionMode |];
                                [| basicBlock.Privilege |] ;
                                BitConverter.GetBytes(basicBlock.LoopCount);
                                basicBlock.Data
                                |]

            let serializeAddressIndependentBasicBlock basicBlock =
                Array.concat [| [| basicBlock.ExecutionMode |]; basicBlock.Data |]

            let isBasicBlockTranslated = traceDataStream.ReadByte ()
            match isBasicBlockTranslated with
            | 0uy ->
                (* basic block is not translated yet *)
                let basicBlockRawSize = traceDataStream.ReadUInt16()
                let basicBlockData = traceDataStream.ReadBytes(int basicBlockRawSize)
                let basicBlockProgramCounter = traceDataStream.ReadUInt64()
                let basicBlockExecutionMode = traceDataStream.ReadByte()
                let basicBlockCPL = traceDataStream.ReadByte()
                let basicBlockLoopCount = traceDataStream.ReadUInt64()

                let basicBlock =
                    { ProgramCounter = basicBlockProgramCounter
                      ExecutionMode = basicBlockExecutionMode
                      Privilege = basicBlockCPL
                      Data = basicBlockData
                      LoopCount = basicBlockLoopCount }

                // ref: https://github.com/StackExchange/StackExchange.Redis/issues/831
                redisDatabase.ListRightPush(~~BasicBlockDataList, ~~basicBlockData) |> ignore

                // ref: https://gist.github.com/theburningmonk/2071722
                let serializedBasicBlock = serializeBasicBlock basicBlock

                // printfn "serialized basic block length: %d" <| Array.length serializedBasicBlock

                redisDatabase.ListRightPush(~~BasicBlockList, ~~serializedBasicBlock) |> ignore

                let serializedAddressIndependentBasicBlock = serializeAddressIndependentBasicBlock basicBlock
                Set.add (hash serializedAddressIndependentBasicBlock) addressIndependentBasicBlockHashes |> ignore
                redisDatabase.ListRightPush(~~AddressIndependentBasicBlockList, ~~serializedAddressIndependentBasicBlock) |> ignore

                // printfn "raw basic block size: %d" <| Array.length basicBlockData
                // printfn "serialized basic block size: %d" <| Array.length serializedBasicBlock

            | 1uy ->
                (* basic block has been translated *)
                let basicBlockIndex = traceDataStream.ReadUInt64()
                let basicBlockData =
                    ~~redisDatabase.ListGetByIndex(~~BasicBlockDataList, int64 basicBlockIndex)
                let basicBlockProgramCounter = traceDataStream.ReadUInt64()
                let basicBlockExecutionMode = traceDataStream.ReadByte()
                let basicBlockCPL = traceDataStream.ReadByte()
                let basicBlockLoopCount = traceDataStream.ReadUInt64()

                let basicBlock =
                    { ProgramCounter = basicBlockProgramCounter
                      ExecutionMode = basicBlockExecutionMode
                      Privilege = basicBlockCPL
                      Data = basicBlockData
                      LoopCount = basicBlockLoopCount }

                let serializedBasicBlock = serializeBasicBlock basicBlock
                redisDatabase.ListRightPush(~~BasicBlockList, ~~serializedBasicBlock) |> ignore

                let serializedAddressIndependentBasicBlock = serializeAddressIndependentBasicBlock basicBlock
                if not (Set.contains (hash serializedAddressIndependentBasicBlock) addressIndependentBasicBlockHashes) then
                    Set.add (hash serializedAddressIndependentBasicBlock) addressIndependentBasicBlockHashes |> ignore
                    redisDatabase.ListRightPush(~~AddressIndependentBasicBlockList, ~~serializedAddressIndependentBasicBlock) |> ignore

            | _ -> failwith "unreachable"

        member gen.RawBasicBlockCount =
            // let inline (~~) (x : ^a) : ^b = ((^a or ^b) : (static member op_Implicit : ^a -> ^b) x)
            redisDatabase.ListLength <| RedisKey.op_Implicit BasicBlockDataList

        member gen.GeneratedBasicBlockCount =
            redisDatabase.ListLength <| RedisKey.op_Implicit BasicBlockList

import Foundation

public struct Block {
    public typealias Statement = [Value]

    public var statements: [Statement]
}

extension TraitID where T == Block {
    static let block = Self(debugLabel: "Block")
}

public extension Trait {
    static func block(_ statements: [Block.Statement]) -> Trait<Block> {
        .init(id: .block) { _ in
            Block(statements: statements)
        }
    }
}

// MARK: - Initialize

public func initializeBlock(_ env: inout Environment) {
    // Block ::= Text
    // TODO: Implement in Wipple code
    env.addConformance(
        derivedTraitID: .text,
        validation: TraitID.block.validation(),
        deriveTraitValue: { value, env in
            "<block>"
        }
    )

    // Block ::= Evaluate
    env.addConformance(
        derivedTraitID: .evaluate,
        validation: TraitID.block.validation(),
        deriveTraitValue: { block, env in
            return { env in
                var result = Value()
                for statement in block.statements {
                    // Evaluate each statement as a list

                    let list = Value(location: statement.first?.location)
                        .add(.list(statement))

                    result = try list.evaluate(&env)
                }

                return result
            }
        }
    )
    
    // Block ::= Macro-Expand
    env.addConformance(
        derivedTraitID: .macroExpand,
        validation: TraitID.block.validation(),
        deriveTraitValue: { block, env in
            return { parameter, replacement, env in
                let statements: [Block.Statement] = try block.statements.map { statement in
                    // Replace each statement as a list
                    
                    let list = Value(location: statement.first?.location)
                        .add(.list(statement))
                    
                    return try list
                        .macroExpand(parameter: parameter, replacement: replacement, &env)
                        .trait(.list, &env)
                }
                
                return Value.new(.block(statements))
            }
        }
    )
}

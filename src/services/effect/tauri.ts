import { Effect, Schema } from "effect"
import { invoke } from "@tauri-apps/api/core"

export class TauriInvokeError {
  readonly _tag = "TauriInvokeError"
  constructor(
    readonly command: string,
    readonly cause: unknown,
  ) {}
}

function invokeToEffect(
  command: string,
  args: Record<string, unknown>,
): Effect.Effect<unknown, TauriInvokeError> {
  return Effect.async<unknown, TauriInvokeError>((resume) => {
    invoke(command, args)
      .then(
        (v) => resume(Effect.succeed(v)),
        (e) => resume(Effect.fail(new TauriInvokeError(command, e))),
      )
  })
}

export function tauri<T>(
  command: string,
  args: Record<string, unknown>,
  schema: Schema.Schema<any, any>,
): Promise<T> {
  return Effect.runPromise(
    invokeToEffect(command, args).pipe(
      Effect.flatMap((raw) =>
        Schema.decodeUnknown(schema)(raw).pipe(
          Effect.mapError((e) => new TauriInvokeError(command, e)),
        ),
      ),
    ),
  ) as Promise<T>
}

export function tauriVoid(
  command: string,
  args: Record<string, unknown>,
): Promise<void> {
  return Effect.runPromise(invokeToEffect(command, args)) as Promise<void>
}

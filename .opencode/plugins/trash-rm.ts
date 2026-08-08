import type { Plugin, PluginInput } from "@opencode-ai/plugin"

/**
 * Intercepts `rm` commands in bash tool calls and rewrites them to `trash-put`
 * (from trash-cli). Strips destructive flags (-r, -f, --recursive, --force)
 * since trash-put is non-destructive by nature.
 *
 * Only rewrites `rm` when it is the command at a statement boundary:
 * line start, or right after `;`, `&&`, `||`, `|`, `(`, or `{`. This avoids
 * corrupting legitimate commands that merely contain the token `rm` as an
 * argument (e.g. `grep rm file`, `ls rm`) and skips nested/subshell forms
 * (`sh -c 'rm ...'`, `eval rm`, `$(rm)`) where a string rewrite would not be
 * safe — the bash permission layer is the real boundary.
 */
export default (async (_input: PluginInput) => {
  return {
    "tool.execute.before": async (input, output) => {
      if (input.tool !== "bash") return

      const args = output.args as any
      let cmd: string | null = null

      if (typeof args === "string") {
        cmd = args
      } else if (args && typeof args === "object" && "command" in args) {
        cmd = args.command
      }

      if (!cmd || typeof cmd !== "string") return

      // Skip nested/subshell contexts where rewriting is unsafe.
      if (/(^|[\s;&|(])(sh|bash)\s+-c\b/.test(cmd) || /\beval\b/.test(cmd)) return

      // `rm` as the command at a statement boundary (line start or after
      // ; / && / || / | / ( / {, path prefix allowed). Not after a plain
      // space mid-line — `grep rm file` must stay untouched.
      const rewritten = cmd.replace(
        /(^|[;&|({]\s*)([^\s;&|({]*\/)?rm(?=[\s]|$)/,
        "$1trash-put"
      )
      const cleaned = rewritten
        .replace(/\s*-{1,2}(?:r|R|f|recursive|force|rf|Rf|rF|RF)\b/g, " ")
        .replace(/\s{2,}/g, " ")
        .trim()

      if (cleaned === cmd) return

      if (typeof args === "string") {
        output.args = cleaned
      } else if (args && typeof args === "object") {
        args.command = cleaned
      }
    }
  }
}) satisfies Plugin

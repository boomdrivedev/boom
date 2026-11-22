// A stand-in for the Messages API, for tests/cap.sh: Claude Code talks to it
// in place of Anthropic's, so the test sends nothing beyond the host. The
// first request of the main loop gets a tool_use of Write on the file that
// API_TARGET names. A request that holds a tool_result, and each request
// outside the main loop, gets the text "done". The server
// writes each request body to API_LOG/<n>.json, and its port to API_PORT.

import { writeFileSync } from "node:fs";
import { join } from "node:path";

const log = process.env.API_LOG;
const target = process.env.API_TARGET;
let count = 0;

/** True where a message of the request holds a tool_result. */
function answersATool(body) {
  return (body.messages ?? []).some(
    (message) =>
      Array.isArray(message.content) &&
      message.content.some((block) => block?.type === "tool_result"),
  );
}

/** The events of a stream that gives `message`. */
function events(message) {
  const out = [
    {
      type: "message_start",
      message: { ...message, content: [], stop_reason: null },
    },
  ];
  message.content.forEach((block, index) => {
    if (block.type === "text") {
      out.push({
        type: "content_block_start",
        index,
        content_block: { type: "text", text: "" },
      });
      out.push({
        type: "content_block_delta",
        index,
        delta: { type: "text_delta", text: block.text },
      });
    } else {
      out.push({
        type: "content_block_start",
        index,
        content_block: { ...block, input: {} },
      });
      out.push({
        type: "content_block_delta",
        index,
        delta: {
          type: "input_json_delta",
          partial_json: JSON.stringify(block.input),
        },
      });
    }
    out.push({ type: "content_block_stop", index });
  });
  out.push({
    type: "message_delta",
    delta: { stop_reason: message.stop_reason, stop_sequence: null },
    usage: { output_tokens: 1 },
  });
  out.push({ type: "message_stop" });
  return out.map((event) => `event: ${event.type}\ndata: ${JSON.stringify(event)}\n\n`).join("");
}

const server = Bun.serve({
  hostname: "127.0.0.1",
  port: 0,
  async fetch(request) {
    const url = new URL(request.url);
    if (request.method !== "POST") {
      return Response.json({});
    }
    const body = await request.json().catch(() => ({}));
    count += 1;
    writeFileSync(
      join(log, `${String(count).padStart(3, "0")}.json`),
      JSON.stringify({ path: url.pathname, body }),
    );
    if (url.pathname.endsWith("/count_tokens")) {
      return Response.json({ input_tokens: 1 });
    }
    if (!url.pathname.endsWith("/v1/messages")) {
      return Response.json({});
    }
    const mainLoop = (body.tools ?? []).some((tool) => tool.name === "Write");
    const content =
      mainLoop && !answersATool(body)
        ? [
            {
              type: "tool_use",
              id: "toolu_cap",
              name: "Write",
              input: { file_path: target, content: "x\n" },
            },
          ]
        : [{ type: "text", text: "done" }];
    const message = {
      id: "msg_cap",
      type: "message",
      role: "assistant",
      model: body.model ?? "claude-sonnet-4-5",
      content,
      stop_reason: content[0].type === "tool_use" ? "tool_use" : "end_turn",
      stop_sequence: null,
      usage: {
        input_tokens: 1,
        output_tokens: 1,
        cache_creation_input_tokens: 0,
        cache_read_input_tokens: 0,
      },
    };
    if (body.stream) {
      return new Response(events(message), {
        headers: { "content-type": "text/event-stream" },
      });
    }
    return Response.json(message);
  },
});

writeFileSync(process.env.API_PORT, String(server.port));

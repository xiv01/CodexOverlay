import { describe, expect, it } from "vitest";
import { formatWindowDuration } from "./formatting";

describe("formatWindowDuration", () => {
  it("uses exact compact labels", () => {
    expect(formatWindowDuration(15)).toBe("15m");
    expect(formatWindowDuration(60)).toBe("1h");
    expect(formatWindowDuration(90)).toBe("1h30m");
    expect(formatWindowDuration(300)).toBe("5h");
    expect(formatWindowDuration(1440)).toBe("1d");
    expect(formatWindowDuration(10080)).toBe("7d");
  });
});

import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";
import App from "./App.vue";

describe("App", () => {
  it("renders the POE2 income tracker title", () => {
    expect(mount(App).get("h1").text()).toBe("POE2 每日通货收益");
  });
});

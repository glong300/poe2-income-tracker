import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";
import App from "./App.vue";

describe("App", () => {
  it("renders the POE2 income tracker title", () => {
    expect(mount(App).get("h1").text()).toBe("POE2 每日通货收益");
  });

  it("shows a pending snapshot summary after a valid submission", async () => {
    const wrapper = mount(App);

    await wrapper.get('[name="quantity-exalted"]').setValue("12");
    await wrapper.get("form").trigger("submit");

    expect(wrapper.text()).toContain("已记录 1 项通货余额");
  });
});

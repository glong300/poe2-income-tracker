import { mount } from "@vue/test-utils";
import { describe, expect, it, vi } from "vitest";
vi.mock("./lib/commands", () => ({
  saveSnapshot: vi.fn().mockResolvedValue(undefined),
  getDailyLedger: vi.fn().mockResolvedValue([]),
  getRealm: vi.fn().mockResolvedValue("international"),
  setRealm: vi.fn().mockResolvedValue(undefined),
  getPriceProviderStatus: vi.fn().mockResolvedValue({ provider: "PoeNinja", availability: "AwaitingSync", message: "国际服行情等待同步" }),
}));
import { setRealm } from "./lib/commands";
import App from "./App.vue";

describe("App", () => {
  it("renders the daily income dashboard title", () => {
    expect(mount(App).get("h1").text()).toContain("每日收益");
  });

  it("shows a pending snapshot summary after a valid submission", async () => {
    const wrapper = mount(App);

    await wrapper.get('[name="quantity-exalted"]').setValue("12");
    await wrapper.get("form").trigger("submit");

    expect(wrapper.text()).toContain("快照已保存：1 项通货余额");
  });

  it("persists a realm change from the dashboard", async () => {
    const wrapper = mount(App);

    await wrapper.get('button[value="china"]').trigger("click");

    expect(setRealm).toHaveBeenCalledWith("china");
  });
});

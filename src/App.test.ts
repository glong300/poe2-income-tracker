import { mount } from "@vue/test-utils";
import { describe, expect, it, vi } from "vitest";
vi.mock("./lib/commands", () => ({
  saveSnapshot: vi.fn().mockResolvedValue(undefined),
  getDailyLedger: vi.fn().mockResolvedValue([]),
  getRealm: vi.fn().mockResolvedValue("international"),
  setRealm: vi.fn().mockResolvedValue(undefined),
  getPriceProviderStatus: vi.fn().mockResolvedValue({ provider: "PoeNinja", availability: "AwaitingSync", message: "国际服行情等待同步" }),
  getWeeklyLedger: vi.fn().mockResolvedValue([]),
  importManualPrices: vi.fn().mockResolvedValue(1),
  getCaptureCandidates: vi.fn().mockResolvedValue([]),
  confirmCaptureCandidate: vi.fn().mockResolvedValue(undefined),
  rejectCaptureCandidate: vi.fn().mockResolvedValue(undefined),
}));
import { importManualPrices, setRealm } from "./lib/commands";
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

    await wrapper.get('button[value="international"]').trigger("click");

    expect(setRealm).toHaveBeenCalledWith("international");
  });

  it("imports a manual price CSV from the pricing workspace", async () => {
    const wrapper = mount(App);

    await wrapper.get('[data-testid="nav-pricing"]').trigger("click");
    await wrapper.get("textarea").setValue("currency_id,value,quoted_in,captured_at\nexalted,12,chaos,2026-09-03T12:00:00+08:00");
    await wrapper.get("form").trigger("submit");

    expect(importManualPrices).toHaveBeenCalled();
  });
});

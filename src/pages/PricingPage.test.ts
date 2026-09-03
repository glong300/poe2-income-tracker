import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";
import PricingPage from "./PricingPage.vue";

describe("PricingPage", () => {
  it("emits manual CSV content for local import", async () => {
    const wrapper = mount(PricingPage, {
      props: { providerStatus: { provider: "CNMarket", availability: "Unavailable", message: "国服行情尚未配置" } },
    });

    await wrapper.get("textarea").setValue("currency_id,value,quoted_in,captured_at\nexalted,12,chaos,2026-09-03T12:00:00+08:00");
    await wrapper.get("form").trigger("submit");

    expect(wrapper.emitted("import-csv")).toEqual([["currency_id,value,quoted_in,captured_at\nexalted,12,chaos,2026-09-03T12:00:00+08:00"]]);
  });
});

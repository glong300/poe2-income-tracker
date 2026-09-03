import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";
import HistoryPage from "./HistoryPage.vue";

describe("HistoryPage", () => {
  it("requests the selected weekly history period", async () => {
    const wrapper = mount(HistoryPage, { props: { rows: [] } });

    await wrapper.get('input[type="date"]').setValue("2026-09-01");
    await wrapper.get('[data-testid="weekly-history"]').trigger("click");

    expect(wrapper.emitted("load-week")).toEqual([["2026-09-01"]]);
  });
});

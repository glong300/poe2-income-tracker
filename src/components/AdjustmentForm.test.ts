import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";
import AdjustmentForm from "./AdjustmentForm.vue";

describe("AdjustmentForm", () => {
  it("emits a validated crafting outflow", async () => {
    const wrapper = mount(AdjustmentForm, { props: { currencies: [{ id: "exalted", name: "崇高石" }] } });

    await wrapper.get('input[name="adjustment-quantity"]').setValue("2");
    await wrapper.get('select[name="adjustment-direction"]').setValue("outflow");
    await wrapper.get('select[name="adjustment-kind"]').setValue("crafting");
    await wrapper.get("form").trigger("submit");

    expect(wrapper.emitted("submit")).toEqual([[{ currencyId: "exalted", quantity: 2, direction: "outflow", kind: "crafting" }]]);
  });
});

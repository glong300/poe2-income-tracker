import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";
import SnapshotForm from "./SnapshotForm.vue";

describe("SnapshotForm", () => {
  it("does not submit a negative currency quantity", async () => {
    const wrapper = mount(SnapshotForm, {
      props: { currencies: [{ id: "exalted", name: "崇高石" }] },
    });

    await wrapper.get('[name="quantity-exalted"]').setValue("-1");
    await wrapper.get("form").trigger("submit");

    expect(wrapper.emitted("submit")).toBeUndefined();
    expect(wrapper.text()).toContain("数量必须是非负整数");
  });
});

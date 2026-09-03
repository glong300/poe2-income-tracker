import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";

import RealmSelector from "./RealmSelector.vue";

describe("RealmSelector", () => {
  it("emits china and updates the current badge when 国服 is selected", async () => {
    const wrapper = mount(RealmSelector, { props: { modelValue: "international" } });

    await wrapper.get('button[value="china"]').trigger("click");

    expect(wrapper.emitted("change")?.[0]).toEqual(["china"]);
    expect(wrapper.get("[data-testid='realm-badge']").text()).toBe("国服");
  });
});

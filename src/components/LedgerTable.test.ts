import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";

import LedgerTable from "./LedgerTable.vue";

describe("LedgerTable", () => {
  it("labels a negative unexplained amount as 未归因", () => {
    const wrapper = mount(LedgerTable, {
      props: {
        rows: [{ currencyId: "exalted", netChange: 2, explainedChange: 4, unattributedChange: -2 }],
      },
    });

    expect(wrapper.text()).toContain("未归因 -2");
  });
});

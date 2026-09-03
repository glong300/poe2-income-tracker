import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";
import CandidateReview from "./CandidateReview.vue";

describe("CandidateReview", () => {
  it("submits reviewed quantities only after an explicit confirmation", async () => {
    const wrapper = mount(CandidateReview, {
      props: {
        candidate: {
          id: 7,
          candidate: {
            realm_hint: "china",
            confidence: 88,
            entries: [{ currency_id: "exalted", quantity: 12 }],
          },
        },
      },
    });

    await wrapper.get('input[name="candidate-7-exalted"]').setValue("14");
    await wrapper.get('[data-testid="confirm-candidate"]').trigger("click");

    expect(wrapper.emitted("confirm")).toEqual([[7, [{ currencyId: "exalted", quantity: 14 }]]]);
  });
});

import { expect, fixture, html } from "@open-wc/testing";
import "../../../../src/views/sbc/device-selection-view.js";
import type { DeviceSelectionView } from "../../../../src/views/sbc/device-selection-view.js";

describe("device-selection-view", () => {
  it("renders and shows loading or error state", async () => {
    const el = await fixture<DeviceSelectionView>(html`
      <device-selection-view></device-selection-view>
    `);

    // Component will either be in loading state, error state (no Tauri in test env),
    // or have loaded devices
    const loading = el.shadowRoot!.querySelector(".loading");
    const error = el.shadowRoot!.querySelector(".error");
    const grid = el.shadowRoot!.querySelector(".devices-grid");

    // One of these should exist
    expect(loading || error || grid).to.exist;
  });

  it("has the correct host styles", async () => {
    const el = await fixture<DeviceSelectionView>(html`
      <device-selection-view></device-selection-view>
    `);

    const styles = getComputedStyle(el);
    expect(styles.display).to.equal("flex");
  });
});

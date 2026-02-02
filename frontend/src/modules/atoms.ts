import { CoreAtoms } from "@/modules/core/core.atoms";
import { GenshinAtoms } from "@/modules/games/genshin/genshin.atoms";
import { HsrAtoms } from "@/modules/games/hsr/hsr.atoms";
import { WuwaAtoms } from "@/modules/games/wuwa/wuwa.atoms";
import { ZzzAtoms } from "@/modules/games/zzz/zzz.atoms";
import { SettingsAtoms } from "@/modules/settings/settings.atoms";

// =============================================================================
// Games Atoms Container
// =============================================================================

class GamesAtoms {
  readonly genshin: GenshinAtoms;
  readonly hsr: HsrAtoms;
  readonly zzz: ZzzAtoms;
  readonly wuwa: WuwaAtoms;

  constructor(core: CoreAtoms) {
    this.genshin = new GenshinAtoms(core);
    this.hsr = new HsrAtoms(core);
    this.zzz = new ZzzAtoms(core);
    this.wuwa = new WuwaAtoms(core);
  }
}

// =============================================================================
// Main Atoms Container
// =============================================================================

class AtomsContainer {
  readonly core: CoreAtoms;
  readonly games: GamesAtoms;
  readonly settings: SettingsAtoms;

  constructor() {
    this.core = new CoreAtoms();
    this.games = new GamesAtoms(this.core);
    this.settings = new SettingsAtoms();
  }
}

export const atoms = new AtomsContainer();

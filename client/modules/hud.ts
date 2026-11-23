import { getEl } from './constants.js';
import type { EstadoJogo } from './types.js';

const hpVal = getEl<HTMLSpanElement>("hp-val");
const luzVal = getEl<HTMLSpanElement>("luz-val");
const somVal = getEl<HTMLSpanElement>("som-val");
const invList = getEl<HTMLUListElement>("inv-list");
const invTitulo = document.querySelector<HTMLElement>("#hud-inv, .inventory-box h3");
const slotsContainer = getEl<HTMLDivElement>('inv-slots-container');

export function atualizarSidebar(estado?: EstadoJogo): void {
    if (!estado) return;

    if (hpVal) {
        if (estado.hp === "∞") {
            hpVal.textContent = "[ GOD MODE ]";
            hpVal.className = "amarelo";
            document.body.classList.remove("hp-critico");
        } else {
            const hpAtual = parseInt(String(estado.hp), 10) || 0;
            const maxHp = 3;
            const blocosCheios = "█".repeat(Math.min(20, Math.max(0, hpAtual)));
            const blocosVazios = "░".repeat(Math.min(maxHp, Math.max(0, maxHp - hpAtual)));

            hpVal.textContent = `[${blocosCheios}${blocosVazios}]`;
            hpVal.className = hpAtual <= 1 ? "vermelho" : "verde";
            document.body.classList.toggle("hp-critico", hpAtual <= 1 && hpAtual > 0);
        }
    }

    if (luzVal) {
        luzVal.textContent = estado.luz_restante !== undefined ? String(estado.luz_restante) : "??";
        const luzAlta = estado.luz_restante === "∞" || Number(estado.luz_restante) > 3;
        luzVal.className = luzAlta ? "verde" : "vermelho";
    }

    if (somVal && estado.som !== undefined) {
        somVal.textContent = `${estado.som}%`;
        if (estado.som >= 80) somVal.className = "vermelho";
        else if (estado.som >= 50) somVal.className = "amarelo";
        else somVal.className = "verde";
    }


    if (invList) {
        invList.textContent = "";

        const qtdBolsas = estado.bolsas_coletadas || 0;
        const limiteMaximo = 3 + (qtdBolsas * 3);
        const inventario = estado.inventario || [];
        const qtdAtual = inventario.length;

        if (invTitulo) {
            invTitulo.textContent = `INV (${qtdAtual}/${limiteMaximo}):`;
        }

        if (qtdAtual > 0) {
            inventario.forEach(item => {
                const li = document.createElement("li");
                li.textContent = `- ${item}`;
                li.className = "branco";
                invList.appendChild(li);
            });
        } else {
            const li = document.createElement("li");
            li.textContent = "Vazio";
            li.className = "amarelo";
            invList.appendChild(li);
        }

        if (slotsContainer) {
            slotsContainer.textContent = "";
            for (let i = 0; i < Math.min(30, limiteMaximo); i++) {
                const slotBox = document.createElement("div");
                slotBox.className = `inv-slot ${i < qtdAtual ? "cheio" : "vazio"}`;
                slotsContainer.appendChild(slotBox);
            }
        }
    }
}

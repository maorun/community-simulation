import init, { run_simulation } from "../pkg/community_simulation.js";

const form = document.querySelector("#simulation-form");
const error = document.querySelector("#error");
const summary = document.querySelector("#summary");
const chart = document.querySelector("#chart");

function drawChart(history) {
  const series = Object.values(history).slice(0, 5);
  const context = chart.getContext("2d");
  context.clearRect(0, 0, chart.width, chart.height);
  if (!series.length) return;
  const values = series.flat();
  const min = Math.min(...values);
  const max = Math.max(...values);
  const range = max - min || 1;
  const colors = ["#1565c0", "#c62828", "#2e7d32", "#6a1b9a", "#ef6c00"];
  series.forEach((prices, index) => {
    context.beginPath();
    context.strokeStyle = colors[index];
    prices.forEach((price, step) => {
      const x = (step / Math.max(prices.length - 1, 1)) * chart.width;
      const y = chart.height - ((price - min) / range) * chart.height;
      step ? context.lineTo(x, y) : context.moveTo(x, y);
    });
    context.stroke();
  });
}

await init();
form.addEventListener("submit", (event) => {
  event.preventDefault();
  error.textContent = "";
  const data = new FormData(form);
  const options = Object.fromEntries(data);
  for (const key of ["steps", "persons", "seed", "initial_money", "base_price"]) {
    options[key] = Number(options[key]);
  }
  for (const input of form.querySelectorAll('input[type="checkbox"]')) {
    options[input.name] = input.checked;
  }
  try {
    const result = JSON.parse(run_simulation(JSON.stringify(options)));
    summary.textContent = JSON.stringify({
      total_steps: result.total_steps,
      active_persons: result.active_persons,
      money_statistics: result.money_statistics,
    }, null, 2);
    drawChart(result.skill_price_history);
  } catch (exception) {
    error.textContent = exception.message;
  }
});

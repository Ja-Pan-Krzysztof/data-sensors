async function loadHomeData() {
    try {
        const response = await fetch('/api/sensors');
        if (!response.ok) throw new Error('Błąd połączenia z modułem ESP32');

        const sensors = await response.json();
        const tbody = document.getElementById('home-table-body');

        if (!tbody) return;

        let htmlContent = '';

        sensors.forEach(s => {
            let isAlarm = false;

            if (s.id === 5) {
                isAlarm = (s.value > 0 && s.value <= s.min_threshold);
            } else if (s.id === 3 || s.id === 4) {
                isAlarm = (s.value === 1.0);
            } else {
                isAlarm = (s.value <= s.min_threshold || s.value >= s.max_threshold);
            }

            const statusHtml = isAlarm
                ? '<span class="status-alarm">ALARM</span>'
                : '<span class="status-ok">OK</span>';

            htmlContent += `
                <tr>
                    <td class="bold">${s.name}</td>
                    <td>${s.value.toFixed(1)}</td>
                    <td>${s.min_threshold}</td>
                    <td>${s.max_threshold}</td>
                    <td>${statusHtml}</td>
                </tr>
            `;
        });

        tbody.innerHTML = htmlContent;
    } catch (error) {
        console.error("Błąd pobierania danych Home: ", error);
    }
}

async function loadSettingsData() {
    try {
        const response = await fetch('/api/sensors');
        if (!response.ok) throw new Error('Connecting error');

        const sensors = await response.json();
        const tbody = document.getElementById('settings-table-body');

        if (!tbody) return;

        let htmlContent = '';

        sensors.forEach(s => {
            htmlContent += `
                <tr>
                    <td class="bold">${s.name}</td>
                    <td>${s.min_threshold}</td>
                    <td>${s.max_threshold}</td>
                    <td><input type="number" id="min-in-${s.id}" value="${s.min_threshold}" step="0.1"></td>
                    <td><input type="number" id="max-in-${s.id}" value="${s.max_threshold}" step="0.1"></td>
                </tr>
            `;
        });

        tbody.innerHTML = htmlContent;
    } catch (error) {
        console.error("Error whilst loading settings: ", error);
    }
}

async function saveAllSettings() {
    try {
        for (let i = 1; i <= 5; i++) {
            const minEl = document.getElementById(`min-in-${i}`);
            const maxEl = document.getElementById(`max-in-${i}`);

            if (minEl && maxEl) {
                const payloadText = `${i},${parseFloat(minEl.value)},${parseFloat(maxEl.value)}`;

                await fetch('/api/settings', {
                    method: 'POST',
                    headers: { 'Content-Type': 'text/plain' },
                    body: payloadText
                });
            }
        }

        alert("Saved settings");
        loadSettingsData();

    } catch (error) {
        alert("Error whilst saving settings");
        console.error(error);
    }
}

document.addEventListener("DOMContentLoaded", () => {
    if (document.getElementById('home-table-body')) {
        loadHomeData();
        setInterval(loadHomeData, 1000);
    }

    if (document.getElementById('settings-table-body')) {
        loadSettingsData();
    }
});
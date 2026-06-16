async function fetchSensorData() {
    try {
        const response = await fetch('/api/data');

        if (!response.ok) throw new Error('Network response error');

        const data = await response.json();

        document.getElementById('temp').innerText = data.temperature.toFixed(2) + " °C";
        document.getElementById('light').innerText = data.light_percent.toFixed(2) + " %";
        document.getElementById('tilt').innerText = data.is_tilted ? "Tilted" : "Stable";
        document.getElementById('vibe').innerText = data.shock_detected ? "Detected" : "None";

        if (data.distance_cm >= 0) {
            document.getElementById('dist').innerText = data.distance_cm.toFixed(1) + " cm";
        } else {
            document.getElementById('dist').innerText = "No obstacles";
        }
    } catch (error) {
        console.error("Fetch error: ", error);
    }
}

setInterval(fetchSensorData, 2000);
window.onload = fetchSensorData;

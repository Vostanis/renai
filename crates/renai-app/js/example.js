document.addEventListener('DOMContentLoaded', function() {
    var ctx = document.getElementById('revenueGraph').getContext('2d');
    new Chart(ctx, {
        // Your Chart.js graph configuration here
        type: 'bar', // Example: 'line', 'bar', etc.
        data: {
            labels: ['January', 'February', 'March', 'April'], // Example labels
            datasets: [{
                label: 'Revenue',
                data: [12000, 19000, 3000, 5000], // Example data
                backgroundColor: 'rgba(255, 99, 132, 0.2)',
                borderColor: 'rgba(255, 99, 132, 1)',
                borderWidth: 1
            }]
        },
        options: {
            scales: {
                y: {
                    beginAtZero: true
                }
            }
        }
    });
});
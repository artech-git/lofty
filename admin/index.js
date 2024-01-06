document.getElementById('fileInput').addEventListener('change', handleFileSelect);

function handleFileSelect(event) {
    const files = event.target.files;
    const fileList = document.getElementById('fileList');
    const progressBars = document.getElementById('progressBars');
    fileList.innerHTML = '';
    progressBars.innerHTML = '';

    for (let i = 0; i < files.length; i++) {
        const file = files[i];
        const li = document.createElement('li');
        li.textContent = `${file.name} (${formatFileSize(file.size)})`;
        fileList.appendChild(li);

        const progressBar = document.createElement('div');
        progressBar.classList.add('progress-bar');
        const progressBarInner = document.createElement('div');
        progressBarInner.classList.add('progressBarInner');
        progressBar.appendChild(progressBarInner);
        progressBars.appendChild(progressBar);

        uploadFile(file, progressBarInner);
    }
}

function uploadFile(file, progressBarInner) {
    const progress = progressBarInner;
    const fileSize = file.size;
    let uploadedSize = 0;

    const interval = setInterval(() => {
        if (uploadedSize >= fileSize) {
            clearInterval(interval);
        } else {
            uploadedSize += 1024; // Simulating progress increment
            const percentage = (uploadedSize / fileSize) * 100;
            progress.style.width = percentage + '%';
        }
    }, 50);
}

function formatFileSize(size) {
    if (size === 0) return '0 Bytes';
    const units = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(size) / Math.log(1024));
    return parseFloat((size / Math.pow(1024, i)).toFixed(2)) + ' ' + units[i];
}

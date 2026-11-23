import { setTimeout as esperar } from 'node:timers/promises';

function obrigatoria(nome) {
    const valor = process.env[nome];
    if (!valor) throw new Error('Configuração ausente: ' + nome);
    return valor;
}

const token = obrigatoria('VERCEL_TOKEN');
const equipe = obrigatoria('VERCEL_ORG_ID');
const projetos = [obrigatoria('VERCEL_BACKEND_PROJECT'), obrigatoria('VERCEL_FRONTEND_PROJECT')];
if (projetos[0] === projetos[1]) throw new Error('Configure dois projetos distintos.');
const gitSource = {
    type: 'github',
    repoId: Number(obrigatoria('GITHUB_REPOSITORY_ID')),
    ref: obrigatoria('GITHUB_REF_NAME'),
    sha: obrigatoria('GITHUB_SHA'),
};

async function solicitar(caminho, corpo) {
    const url = new URL(caminho, 'https://api.vercel.com');
    url.searchParams.set('teamId', equipe);
    const resposta = await fetch(url, {
        method: corpo ? 'POST' : 'GET',
        headers: { Authorization: 'Bearer ' + token, 'Content-Type': 'application/json' },
        body: corpo ? JSON.stringify(corpo) : undefined,
        signal: AbortSignal.timeout(30_000),
    });
    if (!resposta.ok) throw new Error('Vercel respondeu com HTTP ' + resposta.status);
    return resposta.json();
}

for (const nome of projetos) {
    const projeto = await solicitar('/v9/projects/' + encodeURIComponent(nome));
    let deploy = await solicitar('/v13/deployments', {
        project: projeto.id, name: projeto.name, target: 'production', gitSource,
    });
    while (['QUEUED', 'INITIALIZING', 'BUILDING'].includes(deploy.readyState)) {
        await esperar(5000);
        deploy = await solicitar('/v13/deployments/' + encodeURIComponent(deploy.id));
    }
    if (deploy.readyState !== 'READY') throw new Error(nome + ': ' + deploy.readyState);
    console.log(nome + ' pronto em ' + gitSource.sha);
}

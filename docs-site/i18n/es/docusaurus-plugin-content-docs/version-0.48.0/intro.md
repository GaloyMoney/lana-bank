---
id: intro
title: Plataforma Bancaria Lana
slug: /
sidebar_position: 1
---

# Plataforma Bancaria Lana

Lana es una plataforma bancaria central moderna diseñada para operaciones de préstamos digitales y custodia. Proporciona APIs y herramientas integrales para gestionar facilidades crediticias, cuentas de clientes y operaciones financieras.

## Elige tu Ruta

<div className="row">
  <div className="col col--6">
    <div className="card">
      <div className="card__header">
        <h3>Documentación Técnica</h3>
      </div>
      <div className="card__body">
        <p>Procesos de negocio, conceptos de dominio y procedimientos del panel de administración para el personal bancario.</p>
      </div>
      <div className="card__footer">
        <a className="button button--primary button--block" href="technical-documentation/">Documentación Técnica</a>
      </div>
    </div>
  </div>
  <div className="col col--6">
    <div className="card">
      <div className="card__header">
        <h3>Para Desarrolladores Externos</h3>
      </div>
      <div className="card__body">
        <p>Integra con las APIs GraphQL de Lana desde aplicaciones externas.</p>
      </div>
      <div className="card__footer">
        <a className="button button--primary button--block" href="for-external-developers/">Guía para Desarrolladores Externos</a>
      </div>
    </div>
  </div>
</div>

<div className="row" style={{marginTop: '1rem'}}>
  <div className="col col--6">
    <div className="card">
      <div className="card__header">
        <h3>Para Desarrolladores Internos</h3>
      </div>
      <div className="card__body">
        <p>Configuración local, aplicaciones frontend, arquitectura de dominio y patrones de código.</p>
      </div>
      <div className="card__footer">
        <a className="button button--primary button--block" href="for-internal-developers/">Guía para Desarrolladores Internos</a>
      </div>
    </div>
  </div>
  <div className="col col--6">
    <div className="card">
      <div className="card__header">
        <h3>Para Ingenieros de Plataforma</h3>
      </div>
      <div className="card__body">
        <p>Arquitectura de sistemas, implementación, CI/CD y pipelines de datos.</p>
      </div>
      <div className="card__footer">
        <a className="button button--primary button--block" href="for-platform-engineers/">Guía de Plataforma</a>
      </div>
    </div>
  </div>
</div>

## Aspectos Destacados de la Plataforma

| Capacidad | Descripción |
|------------|-------------|
| **Facilidades Crediticias** | Gestión completa del ciclo de vida de préstamos y líneas de crédito |
| **Multimoneda** | Soporte nativo para USD y BTC |
| **Event Sourcing** | Rastro de auditoría completo de todas las operaciones |
| **APIs GraphQL** | API de administración (interna) + API de cliente (externa) |
| **Contabilidad por Partida Doble** | Impulsada por el libro mayor Cala |
| **Arquitectura Hexagonal** | Separación clara de responsabilidades |

## Enlaces Rápidos

### APIs

- [Referencia de la API de Administración](apis/admin-api/) - Operaciones administrativas completas
- [Referencia de la API de Clientes](apis/customer-api/) - Operaciones orientadas al cliente
- [Eventos de Dominio](apis/events/) - Catálogo de eventos

### Operaciones

- [Gestión de Créditos](technical-documentation/credit/) - Ciclo de vida de facilidades
- [Contabilidad](technical-documentation/accounting/) - Operaciones financieras

### Técnico

- [Arquitectura del Sistema](for-platform-engineers/functional-architecture) - Diseño técnico integral
- [Modelos de Datos](for-platform-engineers/erds/) - Diagramas de relación de entidades
- [Desarrollo Local](for-internal-developers/local-development) - Configuración del entorno de desarrollo

## Primeros Pasos

¿Nuevo en Lana? Comienza con la guía de [Primeros Pasos](getting-started/).
